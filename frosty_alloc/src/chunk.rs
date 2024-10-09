use std::alloc::{alloc, Layout};
use std::ptr::{drop_in_place, NonNull};

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

#[derive(Copy, Clone, Debug)]
struct ListNode {
    value: Chunk,
    next: Option<NonNull<ListNode>>,
}

impl ListNode {
    fn new(chunk: Chunk) -> Self {
        Self {
            value: chunk,
            next: None,
        }
    }

    fn heap_alloc(chunk: Chunk) -> NonNull<Self> {
        let layout = Layout::new::<Self>();
        unsafe {
            let c_ptr = (alloc(layout) as *mut Self).as_mut().unwrap();
            c_ptr.value = chunk;
            c_ptr.next = None;
            NonNull::new(c_ptr as *mut Self).unwrap()
        }
    }

    unsafe fn mut_next(&mut self) -> Option<&mut Self> {
        match &mut self.next {
            None => None,
            Some(ptr) => Some(ptr.as_mut()),
        }
    }

    // this is for merging a value who has uninit ptrs
    fn merge_empty_right(&mut self, right: Self) {
        let new_chunk = Chunk {
            start: self.value.start,
            len: self.value.len + right.value.len,
        };
        self.value = new_chunk;
    }

    // this is for mergining a value who has uninit ptrs
    // NOTE: a node can have None for ptrs and still be init
    //       if its a head or tail
    fn merge_right(&mut self, right: Self) {
        let new_chunk = Chunk {
            start: self.value.start,
            len: self.value.len + right.value.len,
        };
        self.value = new_chunk;
        self.next = right.next;
    }

    // no fancy ptr rewiring needs to be done
    // when merging left.
    fn merge_left(&mut self, left: Self) {
        let new_chunk = Chunk {
            start: left.value.start,
            len: self.value.len + left.value.len,
        };
        self.value = new_chunk;
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

    // this method only exists to help with
    // debugging and tests
    #[cfg(test)]
    fn recursive_get_size(&self) -> usize {
        let mut size = 0;
        if self.head.is_none() {
            return 0;
        }
        let mut cur = Some(NonNull::new(&mut self.head.unwrap() as *mut ListNode).unwrap());
        loop {
            match cur {
                None => break,
                Some(mut node) => unsafe {
                    size += 1;
                    cur = node.as_mut().next;
                },
            }
        }
        size
    }

    // TODO:
    //      Move this to happen concurrently with
    //      finding optimal chunk for increased performance
    pub unsafe fn pop_index(&mut self, mut index: isize) -> Chunk {
        if index == 0 {
            let head = self.head.unwrap();
            self.head = match head.next {
                None => None,
                Some(c) => Some(*c.as_ref()),
            };
            return head.value;
        }

        // this will stop at the node prior to the one to pop
        let mut cur = &mut self.head.unwrap();
        while index > 1 {
            cur = cur.next.unwrap().as_mut();
            index -= 1;
        }

        let to_pop = cur.next.unwrap().as_mut();
        cur.next = to_pop.next;
        let c = to_pop.value;
        drop_in_place(to_pop as *mut ListNode);
        c
    }

    pub fn add(&mut self, chunk: Chunk) {
        unsafe {
            let mut cur = match &mut self.head {
                // this is the empty case
                None => {
                    self.head = Some(ListNode::new(chunk));
                    self.len += 1;
                    return;
                }
                Some(head) => head,
            };
            let new_node = ListNode::heap_alloc(chunk);

            // this is the 1 item case
            if new_node.as_ref().value.start + new_node.as_ref().value.len == cur.value.start {
                cur.merge_left(*new_node.as_ref());
            }
            // this is the standard case
            loop {
                // try merging with cur
                if cur.value.start + cur.value.len == new_node.as_ref().value.start {
                    cur.merge_empty_right(*new_node.as_ref());
                    if let Some(next) = &cur.next {
                        if cur.value.start + cur.value.len == next.as_ref().value.start {
                            cur.merge_right(*next.as_ref());
                        }
                    }
                    return;
                }
                let next_clone = cur.next;
                match next_clone {
                    None => {
                        cur.next = Some(new_node);
                        self.len += 1;
                        return;
                    }
                    Some(mut next) => {
                        if new_node.as_ref().value.start + new_node.as_ref().value.len
                            == next.as_ref().value.start
                        {
                            next.as_mut().merge_left(*new_node.as_ref());
                            return;
                        }
                        cur = next.as_mut();
                    }
                }
            }
        }
    }

    // get the [Chunk] which can best fit an [Object] with size [size]
    // and pop it from the list
    pub fn get_best_fit(&mut self, size: usize) -> Option<Chunk> {
        let mut best_fit_index = -1;
        let mut best_fit_value = 0.0;
        let mut cur = match &mut self.head {
            // cannot use ? since [ListNode] doesnt impl Copy
            None => return None,
            Some(val) => val,
        };

        let mut i = 0;
        unsafe {
            loop {
                let fitness = cur.value.calculate_fitness(size);
                if fitness > best_fit_value {
                    best_fit_value = fitness;
                    best_fit_index = i;
                }
                match cur.next {
                    Some(mut node) => {
                        cur = node.as_mut();
                        i += 1;
                    }
                    None => {
                        break;
                    }
                };
            }

            if best_fit_index < 0 {
                return None;
            }
            return Some(self.pop_index(best_fit_index));
        }
    }
}

// Test coverage incomplete
// TODO:
//  Complete test coverage:
//      - Merge header w/ left
//      - Add node predating header
//      - Merge left non-head
//      - Merge 3 nodes
//      - Remove nodes
//      - Remove and readd nodes
#[cfg(test)]
mod chunk_test {
    use super::{Chunk, OrderedChunkList};

    #[test]
    fn get_size() {
        let mut ocl = OrderedChunkList::new();
        assert_eq!(0, ocl.recursive_get_size());
        ocl.add(Chunk { start: 0, len: 20 });
        assert_eq!(1, ocl.recursive_get_size());
        ocl.add(Chunk { start: 21, len: 10 });
        assert_eq!(2, ocl.recursive_get_size());
    }

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
        let c2 = Chunk { start: 21, len: 10 };
        ocl.add(c1);
        ocl.add(c2);
        assert_eq!(2, ocl.len);
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
        let c2 = Chunk { start: 21, len: 10 };
        ocl.add(c1);
        ocl.add(c2);
        let best_fit = ocl.get_best_fit(5).unwrap();
        assert_eq!(best_fit, c2);
    }

    #[test]
    fn merge_right() {
        let mut ocl = OrderedChunkList::new();
        let c1 = Chunk { start: 0, len: 20 };
        let c2 = Chunk { start: 20, len: 10 };
        let expected = Chunk { start: 0, len: 30 };
        ocl.add(c1);
        ocl.add(c2);
        let first = ocl.head.unwrap();
        assert_eq!(expected, first.value);
        assert_eq!(ocl.len, 1);
    }

    #[test]
    fn pop_head_node() {
        let mut ocl = OrderedChunkList::new();
        ocl.add(Chunk { start: 0, len: 20 });
        assert_eq!(1, ocl.recursive_get_size());
        ocl.get_best_fit(5);
        assert_eq!(0, ocl.recursive_get_size());
    }
}
