use std::ptr::NonNull;

#[cfg_attr(test, derive(Eq, PartialEq))]
#[derive(Clone, Copy, Debug)]
pub(crate) struct Chunk {
    pub start: usize,
    pub len: usize,
}

impl Chunk {
    fn calculate_fitness(&self, size: usize) -> f32 {
        // create a score representing how well a sized object
        // fits in a chunk. With values of 1 being a perfect
        // fit and 0 being not fitting
        (size as f32 / self.len as f32) * (self.len >= size) as i32 as f32
    }

    pub fn reduce(&mut self, amnt: usize) {
        self.start += amnt;
        self.len -= amnt;
    }
}

#[derive(Clone, Debug)]
struct ListNode {
    value: Chunk,
    prev: Option<NonNull<ListNode>>,
    next: Option<NonNull<ListNode>>,
}

impl ListNode {
    fn new(chunk: Chunk) -> Self {
        Self {
            value: chunk,
            prev: None,
            next: None,
        }
    }

    unsafe fn mut_next(&self) -> Option<&mut Self> {
        Some(self.next?.as_mut())
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

        // unsafe due to heap allocations
        unsafe {
            loop {
                if cur.next.is_none() {
                    let mut new_node = ListNode::new(chunk);
                    new_node.prev = Some(NonNull::new(cur as *mut ListNode).unwrap());
                    cur.next = Some(NonNull::new(&mut new_node as *mut ListNode).unwrap());
                    return;
                }
                let next = cur.next.as_mut().unwrap().as_mut();
                if next.value.start > chunk.start {
                    let mut new_node = ListNode::new(chunk);
                    let next_node = cur.next.unwrap();
                    new_node.next = Some(next_node);
                    cur.next = Some(NonNull::new(&mut new_node as *mut ListNode).unwrap());
                }
                cur = cur.next.unwrap().as_mut();
            }
        }
    }

    // get the [Chunk] which can best fit an [Object] with size [size]
    // and pop it from the list
    pub fn get_best_fit(&mut self, size: usize) -> Option<Chunk> {
        let mut best_fit: Option<&ListNode> = None;
        let mut best_fit_value = 0.0;
        let mut cur = match &mut self.head {
            // cannot use ? since [ListNode] doesnt impl Copy
            None => return None,
            Some(val) => val,
        };
        unsafe {
            loop {
                let fitness = cur.value.calculate_fitness(size);
                if fitness > best_fit_value {
                    best_fit_value = fitness;
                    best_fit = Some(cur);
                }
                match cur.mut_next() {
                    Some(node) => cur = node,
                    None => {
                        break;
                    }
                };
            }
            match best_fit {
                None => None,
                Some(c) => {
                    // drop node from list
                    let c_mut = (c as *const ListNode as *mut ListNode).as_mut().unwrap();
                    if let Some(prev) = &mut c_mut.prev {
                        prev.as_mut().next = c_mut.next;
                    }
                    if let Some(next) = &mut c_mut.next {
                        next.as_mut().prev = c_mut.prev;
                    }
                    Some(c.value)
                }
            }
        }
    }
}

#[cfg(test)]
mod chunk_test {
    use super::{Chunk, OrderedChunkList};

    #[test]
    fn push_head() {
        let mut ocl = OrderedChunkList::new();
        let chunk = Chunk { start: 0, len: 20 };
        ocl.add(chunk);
    }

    #[test]
    fn push_multiple() {
        let mut ocl = OrderedChunkList::new();
        let c1 = Chunk { start: 0, len: 20 };
        let c2 = Chunk { start: 20, len: 10 };
        ocl.add(c1);
        ocl.add(c2);
    }

    #[test]
    fn get_sized() {
        let mut ocl = OrderedChunkList::new();
        let chunk = Chunk { start: 0, len: 20 };
        ocl.add(chunk);
        let c = ocl.get_best_fit(10).unwrap();
        assert_eq!(chunk, c);
    }

    #[test]
    fn get_over_sized() {
        let mut ocl = OrderedChunkList::new();
        let chunk = Chunk { start: 0, len: 20 };
        ocl.add(chunk);
        let c = ocl.get_best_fit(30);
        assert_eq!(c, None);
    }

    #[test]
    fn get_second_as_best() {
        let mut ocl = OrderedChunkList::new();
        let c1 = Chunk { start: 0, len: 20 };
        let c2 = Chunk { start: 20, len: 10 };
        ocl.add(c1);
        ocl.add(c2);
        let best_fit = ocl.get_best_fit(5).unwrap();
        assert_eq!(best_fit, c2);
    }
}
