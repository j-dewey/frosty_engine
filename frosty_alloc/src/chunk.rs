struct Chunk {
    start: usize,
    len: usize,
}

struct ChunkNode {
    height: i32,
    chunk: Chunk,
}

impl ChunkNode {
    fn check_balance(left: &Self, right: &Self) -> i32 {
        left.height - right.height
    }
}

// AVL style tree stored like a heap tree
pub(crate) struct ChunkTree {
    nodes: Vec<ChunkNode>,
}

// utilities
impl ChunkTree {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(capacity),
        }
    }

    // will
    pub fn push_chunk(&mut self, start: usize, len: usize) {
        let chunk = Chunk { start, len };
        self.rec_push(chunk, 0)
    }
}

// navigation
impl ChunkTree {
    pub fn rec_push(&mut self, chunk: Chunk, index: usize) {
        let node = self.nodes.get_mut(index);
        if node.is_none() {
            self.nodes[index] = ChunkNode { height: 0, chunk };
            return;
        }
        let node = node.unwrap();
    }
}
