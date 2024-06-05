use std::ptr;

use crate::{
    chunk::{Chunk, OrderedChunkList},
    FrostyAllocatable,
};

pub(crate) struct Allocator {
    chunks: OrderedChunkList,
    region: Vec<u8>,
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
        Self { chunks, region }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        let region = Vec::with_capacity(capacity);
        let major_chunk = Chunk {
            start: 0,
            len: region.capacity(),
        };
        let mut chunks = OrderedChunkList::new();
        chunks.add(major_chunk);
        Self { chunks, region }
    }

    pub fn alloc<T: FrostyAllocatable>(&mut self, obj: T) -> Result<(), ()> {
        let size = std::mem::size_of::<T>();
        let chunk = match self.chunks.get_best_fit(size) {
            Some(c) => c,
            None => {
                // increase capacity, this is pretty bad for obvious reasons
                // SystemVec<> will be created to avoid this
                todo!()
            }
        };
        unsafe {
            let init_ptr = self.region.get_mut(chunk.start).unwrap() as *const u8;
            ptr::write(init_ptr as *mut T, obj);
        }
        Ok(())
    }
}
