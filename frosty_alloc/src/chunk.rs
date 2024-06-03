#[derive(Clone, Copy, Debug)]
pub struct Chunk {
    pub start: usize,
    pub len: usize,
}

#[derive(Clone, Debug)]
struct ListNode {
    value: Option<Chunk>,
    next: Option<Box<ListNode>>,
}

impl ListNode {
    fn new(chunk: Chunk) -> Self {
        Self {
            value: Some(chunk),
            next: None,
        }
    }
}

// orders chunks based on their position
pub struct OrderedChunkList {
    head: ListNode,
    len: usize,
}

impl OrderedChunkList {
    pub fn new() -> Self {
        Self {
            head: ListNode {
                value: None,
                next: None,
            },
            len: 0,
        }
    }

    pub fn add(&mut self, chunk: Chunk) {
        self.len += 1;
        let mut cur = &mut self.head;
        loop {
            if cur.value.is_none() {
                cur.value = Some(chunk);
                return;
            }
            if cur.next.is_none() {
                cur.next = Some(Box::new(ListNode::new(chunk)));
                return;
            }
            let next = cur.next.as_mut().unwrap();
            if next.value.is_some() && next.value.unwrap().start > chunk.start {
                let mut new_node = ListNode::new(chunk);
                let next_node = cur.next.as_ref().unwrap().as_ref().clone();
                new_node.next = Some(Box::from(next_node));
                cur.next = Some(Box::new(new_node));
            }
            cur = cur.next.as_mut().unwrap();
        }
    }
}

#[cfg(test)]
mod chunk_list_tests {
    use super::*;

    #[test]
    fn push_item() {
        let mut list = OrderedChunkList::new();
        list.add(Chunk { start: 0, len: 100 });
        assert_eq!(list.len, 1);
    }
}
