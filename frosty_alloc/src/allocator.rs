use std::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

use crate::{
    chunk::{Chunk, OrderedChunkList},
    frosty_box::FrostyBox,
    interim::InterimPtr,
    FrostyAllocatable, ObjectHandle, ObjectHandleMut,
};

pub type Index = usize;

// A simple object that takes control of a region in memory
// which is used to store [Entity]s and [Component]s. This
// is done to provide more control over how they're stored,
// keep them in close proximity, and to make them persist
// across frame updates.
//
// This object does not keep track of where objects are
// stored in its region. Data passed in is stored in a
// [FrostyBox], the address of which is returned to the
// user. When given an index, the [Allocator] assumes that
// it is given a valid address and reads whatever is written
// there. When memory is requested to be free'd, it also
// assumes a valid address is given and frees it.
pub struct Allocator {
    chunks: OrderedChunkList,
    region: Vec<u8>,
    interim: Vec<InterimPtr>,
}

impl Allocator {
    pub fn new() -> Self {
        let mut region = Vec::with_capacity(1);
        // need to init data
        region.push(0);
        let major_chunk = Chunk { start: 0, len: 1 };
        let mut chunks = OrderedChunkList::new();
        chunks.add(major_chunk);
        Self {
            chunks,
            region,
            interim: Vec::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let mut region = Vec::with_capacity(capacity);
        // using region.fill(0) does not properly init data
        for _ in 0..capacity {
            region.push(0);
        }
        let major_chunk = Chunk {
            start: 0,
            len: region.capacity(),
        };
        let mut chunks = OrderedChunkList::new();
        chunks.add(major_chunk);
        Self {
            chunks,
            region,
            interim: Vec::new(),
        }
    }

    // increases capacity of region and returns
    // the previous capacity
    unsafe fn resize(&mut self) -> usize {
        let old_len = self.region.len();
        self.region.reserve(self.region.capacity() * 2);
        // need to init memory
        // TODO:
        //      is there some built-in that allows for better SIMD?
        for _ in old_len..self.region.capacity() {
            self.region.push(0);
        }
        for inter in &mut self.interim {
            let data_start = self.region.get_unchecked_mut(inter.index);
            let ptr = data_start as *mut u8;
            inter.data = NonNull::new(ptr).unwrap();
        }
        old_len
    }

    // Returns index into Interim vec
    pub fn alloc<T: FrostyAllocatable>(&mut self, obj: T) -> Result<ObjectHandleMut<T>, ()> {
        let size = std::mem::size_of::<FrostyBox<T>>();
        let mut chunk = match self.chunks.get_best_fit(size) {
            Some(c) => c,
            None => unsafe {
                // increase capacity, this is pretty bad for obvious reasons
                // SystemVec<> will be created to avoid this
                let old_len = self.resize();
                Chunk {
                    start: old_len,
                    len: self.region.capacity() - old_len,
                }
            },
        };

        let boxed_obj = FrostyBox::new(obj);
        let data_index = chunk.start;
        let interim = unsafe {
            let init_ptr = self.region.get_unchecked_mut(chunk.start) as *mut u8;
            ptr::write_unaligned(init_ptr as *mut FrostyBox<T>, boxed_obj);
            InterimPtr {
                freed: false,
                active_handles: 0,
                data: NonNull::new(init_ptr as *mut u8).unwrap(),
                index: data_index,
            }
        };

        chunk.reduce(size);
        if chunk.len > 0 {
            self.chunks.add(chunk);
        }

        self.interim.push(interim);
        let interim_index = self.interim.len() - 1;
        Ok(ObjectHandleMut {
            ptr: NonNull::new(
                self.interim
                    .get_mut(interim_index)
                    .expect("Allocator Interim Vec has invalid size")
                    as *mut InterimPtr,
            )
            .expect("Failed to create NonNull interim Pointer"),
            _pd: PhantomData,
        })
    }

    pub fn alloc_raw<T: FrostyAllocatable>(&mut self, data: *const T) -> Result<Index, ()> {
        let size = std::mem::size_of::<FrostyBox<T>>();
        let mut chunk = match self.chunks.get_best_fit(size) {
            Some(c) => c,
            None => unsafe {
                // increase capacity, this is pretty bad for obvious reasons
                // SystemVec<> will be created to avoid this
                let old_len = self.resize();
                Chunk {
                    start: old_len,
                    len: self.region.capacity() - old_len,
                }
            },
        };

        let data_index = chunk.start;
        let interim = unsafe {
            // create a frostybox
            let boxed_data: FrostyBox<T> = FrostyBox::from_raw(data);
            // load that box
            let init_ptr = self.region.get_unchecked_mut(chunk.start) as *mut u8;
            ptr::write_unaligned(init_ptr as *mut FrostyBox<T>, boxed_data);
            InterimPtr {
                freed: false,
                active_handles: 0,
                data: NonNull::new(init_ptr as *mut u8).unwrap(),
                index: data_index,
            }
        };

        chunk.reduce(size);
        if chunk.len > 0 {
            self.chunks.add(chunk);
        }

        self.interim.push(interim);
        Ok(self.interim.len() - 1)
    }

    // since the region is completely controlled by [Allocator], the
    // data is free if we say it is. If data has any important Drop
    // functionality, that should be taken care of before free() is
    // called
    // The [ObjectHandle] passed in isn't dropped immediatly. Due to
    // [InterimPtr] being free'd, the handle will no longer be able
    // to access the data
    pub fn free<T: FrostyAllocatable>(&mut self, obj: &mut ObjectHandle<T>) {
        let ptr = obj.get_mut();
        let size = std::mem::size_of::<FrostyBox<T>>();
        let freed_chunk = Chunk {
            start: ptr.index,
            len: size,
        };
        ptr.free();
        self.chunks.add(freed_chunk);
    }

    pub unsafe fn get<T: FrostyAllocatable>(&mut self, index: Index) -> Option<ObjectHandle<T>> {
        let interim = self.interim.get_mut(index)?;
        interim.active_handles += 1;
        Some(ObjectHandle {
            ptr: NonNull::new(interim as *mut InterimPtr).unwrap(),
            _pd: PhantomData {},
        })
    }

    pub fn get_mut<T: FrostyAllocatable>(&mut self, index: Index) -> Option<ObjectHandleMut<T>> {
        let interim = self.interim.get_mut(index)?;
        interim.active_handles += 1;
        Some(ObjectHandleMut {
            ptr: NonNull::new(interim as *mut InterimPtr).unwrap(),
            _pd: PhantomData {},
        })
    }
}

#[cfg(test)]
mod allocator_tests {
    use crate::{AllocId, FrostyAllocatable};

    use super::Allocator;

    struct UniformDummy {
        a: i32,
        b: i32,
    }

    struct NonUniformDummy {
        a: i32,
        b: u8,
    }

    unsafe impl FrostyAllocatable for UniformDummy {
        fn id() -> crate::AllocId
        where
            Self: Sized,
        {
            AllocId::new(10000)
        }
    }

    unsafe impl FrostyAllocatable for NonUniformDummy {
        fn id() -> crate::AllocId
        where
            Self: Sized,
        {
            AllocId::new(10001)
        }
    }

    #[test]
    fn allocate_primitive() {
        let mut alloc = Allocator::with_capacity(4 * 3);
        let data1 = 16;
        let data2 = 16.0;
        let data3 = 16u32;
        let _ = alloc.alloc(data1).unwrap();
        let _ = alloc.alloc(data2).unwrap();
        let _ = alloc.alloc(data3).unwrap();
    }

    #[test]
    fn allocate_uniform_struct() {
        let mut alloc = Allocator::with_capacity(std::mem::size_of::<UniformDummy>());
        let dummy = UniformDummy { a: 10, b: 10 };
        alloc.alloc(dummy).unwrap();
    }

    #[test]
    fn allocate_nonuniform_struct() {
        let mut alloc = Allocator::with_capacity(std::mem::size_of::<NonUniformDummy>());
        let dummy = NonUniformDummy { a: 10, b: 10 };
        alloc.alloc(dummy).unwrap();
    }

    #[test]
    fn access_primitive() {
        let mut alloc = Allocator::with_capacity(4 * 3);
        let data1 = 16;
        let data2 = 16u32;
        let data3 = 2.0;
        let d1i = alloc.alloc(data1).unwrap();
        let d2i = alloc.alloc(data2).unwrap();
        let d3i = alloc.alloc(data3).unwrap();
    }
}
