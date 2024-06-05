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
                todo!()
            }
        };
        todo!()
    }
}
