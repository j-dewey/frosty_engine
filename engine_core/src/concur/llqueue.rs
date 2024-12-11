use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

/*
 * This creates a multiple producer, single consumer lockless queue
 * It is analagous to the standard libraries sync channels, except
 * that if no data has been sent the queue will return None.
 *
 * V Head                V Lag Tail
 * --------------------------------------------------------
 * | Complete | Complete | Partialy | Partialy |          |
 * |   init   |   init   |   init   |   init   |          |
 * --------------------------------------------------------
 *                                             ^ Tail
 */

pub struct QueueFull;
pub struct QueueLost;
pub enum LLQueueError {
    Full(QueueFull),
    Lost(QueueLost),
}

impl From<QueueFull> for LLQueueError {
    fn from(value: QueueFull) -> Self {
        Self::Full(value)
    }
}

// An object able to push onto a lockless queue
pub struct LLPusher<T: Copy + Clone> {
    queue: *mut LLQueueInner<T>,
}

impl<T: Copy + Clone> LLPusher<T> {
    pub fn push(&mut self, item: T) -> Result<(), LLQueueError> {
        unsafe {
            let queue = match self.queue.as_mut() {
                Some(ptr) => ptr,
                None => return Err(LLQueueError::Lost(QueueLost)),
            };
            queue.push(item)?;
            Ok(())
        }
    }
}

// An object able to both push to and pop from a lockless queue
pub struct LLAccess<T: Copy + Clone> {
    queue: *mut LLQueueInner<T>,
}

impl<T: Copy + Clone> LLAccess<T> {
    pub fn push(&mut self, item: T) -> Result<(), LLQueueError> {
        unsafe {
            let queue = match self.queue.as_mut() {
                Some(ptr) => ptr,
                None => return Err(LLQueueError::Lost(QueueLost)),
            };
            queue.push(item)?;
            Ok(())
        }
    }

    pub fn pop(&mut self) -> Option<T> {
        unsafe {
            let queue = self.queue.as_mut()?;
            queue.pop()
        }
    }
}

// A basic lockless queue implementation
pub struct LLQueueInner<T: Copy + Clone> {
    data: Vec<T>,
    head: AtomicUsize, // should this be atomic?
    tail: AtomicUsize,
    lag_tail: AtomicUsize,
    reallocating: AtomicBool,
}

impl<T: Copy + Clone> LLQueueInner<T> {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            head: AtomicUsize::new(0),
            tail: AtomicUsize::new(0),
            lag_tail: AtomicUsize::new(0),
            reallocating: AtomicBool::new(false),
        }
    }

    // This can only be accessed by whichever thread owns the queue
    // which is why it can afford to be slower
    pub fn pop(&mut self) -> Option<T> {
        let old_head = self.head.load(Ordering::Acquire);
        let old_tail = self.lag_tail.load(Ordering::Acquire);
        if old_head == old_tail {
            return None;
        }
        self.head.fetch_add(1, Ordering::SeqCst);

        Some(*self.data.get(old_head)?)
    }

    // This can be accessed by any thread with an accessor, so
    // need to be more cognizant of what data is available
    pub fn push(&mut self, item: T) -> Result<(), QueueFull> {
        let reserved_index = self.tail.fetch_add(1, Ordering::Acquire);
        if reserved_index >= self.data.len() {
            return Err(QueueFull);
        }
        self.data[reserved_index] = item;

        // need to wait for any concurrent but earlier pushes to finalize
        'wait_on_lag_tail: loop {
            let lag_tail = self.lag_tail.load(Ordering::Acquire);
            if lag_tail == reserved_index {
                self.lag_tail.fetch_add(1, Ordering::Acquire);
                break 'wait_on_lag_tail;
            }
        }
        Ok(())
    }
}
