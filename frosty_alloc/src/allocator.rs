use std::collections::LinkedList;

struct Chunk {
    start: usize,
    len: usize,
}

pub(crate) struct Allocator {
    chunks: LinkedList<Chunk>,
}
