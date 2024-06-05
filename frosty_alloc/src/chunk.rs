#[derive(Clone, Copy, Debug)]
pub(crate) struct Chunk {
    pub start: usize,
    pub len: usize,
}

impl Chunk {
    fn calculate_fitness(&self, size: usize) -> f32 {
        // create a score representing how well a sized object
        // fits in a chunk. An object too big returns a value of 0
        (size as f32 / self.len as f32) * (self.len >= size) as i32 as f32
    }
}

#[derive(Clone, Debug)]
struct ListNode {
    value: Chunk,
    next: Option<Box<ListNode>>,
}

impl ListNode {
    fn new(chunk: Chunk) -> Self {
        Self {
            value: chunk,
            next: None,
        }
    }
}

// orders chunks based on their position
pub(crate) struct OrderedChunkList {
    head: Option<ListNode>,
    len: usize,
}

impl OrderedChunkList {
    pub fn new() -> Self {
        Self { head: None, len: 0 }
    }

    pub fn add(&mut self, chunk: Chunk) {
        self.len += 1;

        let mut cur = match &mut self.head {
            None => {
                self.head = Some(ListNode::new(chunk));
                return;
            }
            Some(head) => head,
        };

        loop {
            if cur.next.is_none() {
                cur.next = Some(Box::new(ListNode::new(chunk)));
                return;
            }
            let next = cur.next.as_mut().unwrap();
            if next.value.start > chunk.start {
                let mut new_node = ListNode::new(chunk);
                let next_node = cur.next.as_ref().unwrap().as_ref().clone();
                new_node.next = Some(Box::from(next_node));
                cur.next = Some(Box::new(new_node));
            }
            cur = cur.next.as_mut().unwrap();
        }
    }

    pub fn get_best_fit(&self, size: usize) -> Option<Chunk> {
        // removes from list
        let mut best_fit: Option<Chunk> = None;
        let mut best_fit_value = 0.0;
        let cur = match &self.head {
            // cannot use ? since [ListNode] doesnt impl Copy
            None => return None,
            Some(val) => val,
        };
        'a: loop {
            let cur = match &cur.next {
                Some(node) => node,
                None => {
                    break 'a;
                }
            };
            if cur.value.calculate_fitness(size) > best_fit_value {
                best_fit_value = cur.value.calculate_fitness(size);
                best_fit = Some(cur.value);
            }
        }
        best_fit
    }
}
