use std::ptr;

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
        let region = Vec::new();
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
    fn resize(&mut self) -> usize {
        let old_len = self.region.len();
        self.region.reserve(old_len * 2);
        todo!("Pointer updates for after resize unimplemented");
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let region = Vec::with_capacity(capacity);
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

    pub fn alloc<T: FrostyAllocatable>(&mut self, obj: T) -> Result<Index, ()> {
        let size = std::mem::size_of::<FrostyBox<T>>();
        let mut chunk = match self.chunks.get_best_fit(size) {
            Some(c) => c,
            None => {
                // increase capacity, this is pretty bad for obvious reasons
                // SystemVec<> will be created to avoid this
                let old_len = self.resize();
                Chunk {
                    start: old_len,
                    len: self.region.capacity() - old_len,
                }
            }
        };
        let data_index = chunk.start;
        unsafe {
            let init_ptr = self.region.get_mut(chunk.start).unwrap() as *const u8;
            ptr::write(init_ptr as *mut T, obj);
        }
        chunk.reduce(size);
        if chunk.len > 0 {
            self.chunks.add(chunk);
        }
        Ok(data_index)
    }

    // since the region is completely controlled by [Allocator], the
    // data is free if we say it is. If data has any important Drop
    // functionality, that should be taken care of before free() is
    // called
    pub fn free<T: FrostyAllocatable>(&mut self, index: Index) {
        let size = std::mem::size_of::<FrostyBox<T>>();
        let freed_chunk = Chunk {
            start: index,
            len: size,
        };
        self.chunks.add(freed_chunk);
    }

    pub fn get<T: FrostyAllocatable>(&self, index: Index) -> ObjectHandle<T> {
        todo!()
    }

    pub fn get_mut<T: FrostyAllocatable>(&mut self, index: Index) -> ObjectHandleMut<T> {
        todo!()
    }
}
