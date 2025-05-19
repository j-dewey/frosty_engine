use std::{
    io::Write,
    marker::PhantomData,
    ptr::{self, NonNull},
};

use hashbrown::HashMap;

use crate::{
    chunk::{Chunk, OrderedChunkList},
    debug::DebugOutter,
    frosty_box::FrostyBox,
    group::AllocGroup,
    interim::InterimPtr,
    FrostyAllocatable, ObjectHandle, ObjectHandleMut,
};

// Alliases
pub type Index = usize;
type HeapAlloc<T> = Box<T>;

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
    interim: Vec<HeapAlloc<InterimPtr>>,
}

impl Allocator {
    pub fn new() -> Self {
        Self::with_capacity(4)
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
    unsafe fn resize(&mut self, min_len: usize) -> usize {
        let old_len = self.region.len();
        self.region.reserve(self.region.capacity() * 2 + min_len);
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

    fn get_chunk(&mut self, size: usize) -> Chunk {
        match self.chunks.get_best_fit(size) {
            Some(c) => c,
            None => unsafe {
                // increase capacity, this is pretty bad for obvious reasons
                // SystemVec<> will be created to avoid this
                let old_len = self.resize(size);
                Chunk {
                    start: old_len,
                    len: self.region.capacity() - old_len,
                }
            },
        }
    }

    fn return_chunk(&mut self, mut chunk: Chunk, used: usize) {
        chunk.reduce(used);
        if chunk.len > 0 {
            self.chunks.add(chunk);
        }
    }

    fn get_last_handle(&mut self) -> Result<ObjectHandleMut<u8>, ()> {
        let last_interim = self.interim.last_mut().ok_or(())?;
        Ok(ObjectHandleMut {
            ptr: NonNull::new(last_interim.as_mut() as *mut InterimPtr)
                .expect("failed to get last handle of interim"),
            _pd: PhantomData {},
        })
    }

    // Returns index into Interim vec
    pub fn alloc<T: FrostyAllocatable>(&mut self, obj: T) -> Result<ObjectHandleMut<T>, ()> {
        let size = std::mem::size_of::<FrostyBox<T>>();
        let chunk = self.get_chunk(size);

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

        self.return_chunk(chunk, size);

        self.interim.push(HeapAlloc::new(interim));
        Ok(self.get_last_handle()?.cast_clone())
    }

    pub fn alloc_raw<T: FrostyAllocatable>(
        &mut self,
        data: *const T,
    ) -> Result<ObjectHandleMut<u8>, ()> {
        let size = std::mem::size_of::<FrostyBox<T>>();
        let chunk = self.get_chunk(size);

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

        self.return_chunk(chunk, size);

        self.interim.push(HeapAlloc::new(interim));
        self.get_last_handle()
    }

    // Create a FrostyBox of unknown type and load data into it
    pub unsafe fn alloc_dissolved(&mut self, data: &[u8]) -> Result<ObjectHandleMut<u8>, ()> {
        // 1) Secure a region of memory with enough room
        // 2) Set up a FrostyBox<u8> in that region
        // 3) Change that to a FrostyBox<[u8]>
        // 4) Load data into it
        let raw_size = data.len() + std::mem::size_of::<FrostyBox<u8>>();
        let aligned_size = (raw_size / 4 + 1) * 4;
        let chunk = self.get_chunk(aligned_size);

        let interim = unsafe {
            let uninit_ptr =
                self.region.get_unchecked_mut(chunk.start) as *mut u8 as *mut FrostyBox<u8>;
            let basic_box = FrostyBox::new(0u8);
            ptr::write_unaligned(uninit_ptr, basic_box);
            let data_ptr = uninit_ptr.as_mut().unwrap().get_raw();
            let data_as_slice = std::slice::from_raw_parts_mut(data_ptr, data.len());
            data_as_slice.copy_from_slice(data);
            InterimPtr {
                freed: false,
                active_handles: 0,
                data: NonNull::new(uninit_ptr as *mut u8).unwrap(),
                index: chunk.start,
            }
        };

        self.return_chunk(chunk, aligned_size);

        self.interim.push(Box::new(interim));
        self.get_last_handle()
    }

    // Allocate all objects in a group and connect all
    pub fn alloc_group(&mut self, mut group: AllocGroup) -> Vec<ObjectHandleMut<u8>> {
        let mut indices = Vec::with_capacity(group.objs.len());
        let mut id_to_indx = HashMap::new();
        // allocation
        for (i, (id, data)) in group.objs.drain(..).enumerate() {
            let indx = unsafe {
                self.alloc_dissolved(&data[..])
                    .expect("Failed to alloc dissolved object in AllocGroup")
            };
            indices.push(indx);
            id_to_indx.insert(id, i);
        }
        // set handles
        for (id, needed_ids, setter_fn) in group.handle_set_fns.drain(..) {
            let mut handles: Vec<ObjectHandleMut<u8>> = Vec::new();
            for needed_id in needed_ids {
                let indx = indices.get(*id_to_indx.get(&needed_id).unwrap()).unwrap();
                handles.push(indx.clone());
            }
            let obj_handle = indices.get_mut(*id_to_indx.get(&id).unwrap()).unwrap();
            (setter_fn)(obj_handle.clone(), handles);
        }
        indices
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
        let interim = self.interim.get_mut(index)?.as_mut();
        interim.active_handles += 1;
        Some(ObjectHandle {
            ptr: NonNull::new(interim as *mut InterimPtr).unwrap(),
            _pd: PhantomData {},
        })
    }

    pub fn get_mut<T: FrostyAllocatable>(&mut self, index: Index) -> Option<ObjectHandleMut<T>> {
        let interim = self.interim.get_mut(index)?.as_mut();
        interim.active_handles += 1;
        Some(ObjectHandleMut {
            ptr: NonNull::new(interim as *mut InterimPtr).unwrap(),
            _pd: PhantomData {},
        })
    }
}

impl DebugOutter for Allocator {
    fn dump_data(&self, fs: &mut std::fs::File) {
        // --------------------------------------
        // Allocator
        //      Size       : {}
        //      RegionStart: {}
        //      Interim     : [
        //          <freed: {}, active_handles: {}, index: {}, ptr: {}>
        //      ]

        fs.write_all(b"---------------------------------------------\n")
            .unwrap();
        fs.write_all(b"Allocator\n").unwrap();
        fs.write_all(
            format!(
                "\t Size: {:?}\n\t RegionStart: {:?}\n\t Interim: [\n",
                self.region.len(),
                self.region.as_ptr()
            )
            .as_bytes(),
        )
        .unwrap();
        for int in &self.interim {
            fs.write_all(
                format!(
                    "\t\t <freed: {:?}, active_handles: {:?}, index: {:?}, ptr: {:?}>\n",
                    int.freed, int.active_handles, int.index, int.data
                )
                .as_bytes(),
            )
            .unwrap();
        }
        fs.write_all(b"\t ]\n").unwrap();
    }
}

#[cfg(test)]
mod allocator_tests {

    use crate::{FrostyAllocatable, ObjectHandle};

    use super::Allocator;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct UniformDummy {
        a: i32,
        b: i32,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct NonUniformDummy {
        a: i32,
        b: u8,
    }

    impl UniformDummy {
        pub fn to_bytes(self) -> Box<[u8]> {
            unsafe {
                let mut boxed = Box::new(self); // put in box to heap alloc
                let ptr = boxed.as_mut() as *mut Self as *mut [u8; 8];
                Box::from(&(*ptr)[..])
            }
        }
    }
    unsafe impl FrostyAllocatable for UniformDummy {}

    impl NonUniformDummy {
        pub fn to_bytes(self) -> Box<[u8]> {
            unsafe {
                let mut boxed = Box::new(self); // put in box to heap alloc
                let ptr = boxed.as_mut() as *mut Self as *mut [u8; 5];
                Box::from(&(*ptr)[..])
            }
        }
    }
    unsafe impl FrostyAllocatable for NonUniformDummy {}

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
        let _ = alloc.alloc(data1).unwrap();
        let _ = alloc.alloc(data2).unwrap();
        let _ = alloc.alloc(data3).unwrap();
    }

    #[test]
    fn alloc_without_resize_new() {
        let mut alloc = Allocator::new();
        alloc.alloc(1u8).unwrap();
    }

    #[test]
    fn alloc_with_resize_new() {
        let mut alloc = Allocator::new();
        alloc.alloc(1u128).unwrap();
    }

    #[test]
    fn alloc_non_pod() {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct PointedData {
            data: u8,
        }
        struct PointsToData {
            data: Vec<PointedData>,
        }
        unsafe impl FrostyAllocatable for PointsToData {}

        let mut alloc = Allocator::new();
        let mut ptr = {
            let data = PointedData { data: 10 };
            let ptr = PointsToData { data: vec![data] };

            alloc.alloc(ptr).expect("Failed to alloc PointsToData")
        };
        ptr.get_access_mut(0)
            .expect("Failed to access PointsToData")
            .as_mut()
            .data[0] = PointedData { data: 5 };

        assert_eq!(
            PointedData { data: 5 },
            ptr.get_access(0)
                .expect("Failed to acces PointsToData twice")
                .as_ref()
                .data[0]
        );
    }

    #[test]
    fn alloc_dissolved() {
        let uniform = UniformDummy { a: 1, b: 2 };
        let uniform_bytes = uniform.to_bytes();
        let nonuniform = NonUniformDummy { a: 1, b: 2 };
        let nonuniform_bytes = nonuniform.to_bytes();

        let mut alloc = Allocator::new();
        unsafe {
            let mut uniform_handle = alloc
                .alloc_dissolved(&uniform_bytes[..])
                .expect("Failed to allocate dissolved UniformDummy")
                .cast_clone::<UniformDummy>();

            assert_eq!(
                *uniform_handle.get_access(0).as_ref().unwrap().as_ref(),
                uniform
            );

            let mut nonuniform_handle = alloc
                .alloc_dissolved(&nonuniform_bytes[..])
                .expect("Failed to allocated dissolved NonUniformDummy")
                .cast_clone::<NonUniformDummy>();

            assert_eq!(
                *nonuniform_handle.get_access(0).as_ref().unwrap().as_ref(),
                nonuniform
            );
        }
    }
}
