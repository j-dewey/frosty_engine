use std::sync::atomic::{AtomicU32, Ordering};

// Meta data to be associated with an object

// with 32 bits:
//      2 are reserved for writing flags
//      15 are
pub type BitMaskType = u32;
pub(crate) struct BitMask(pub AtomicU32);

impl BitMask {
    pub const WRITE_FLAG: BitMaskType = 0b10_000000000000000_000000000000000;
    // any value greater than or equal to this is locked to new reads
    pub const LOCK_VALUE: BitMaskType = 0b00_000000000000001_000000000000000;
    pub const NON_READ_FLAGS: BitMaskType = 0b11_111111111111111_000000000000000;
    pub fn new(v: u32) -> Self {
        Self(AtomicU32::new(v))
    }

    pub fn generate_pending_flag(thread: BitMaskType) -> BitMaskType {
        Self::LOCK_VALUE << thread
    }

    // no return value. since this method is blocking,
    // code execution begins again once access is granted
    pub fn get_access(&mut self, thread: BitMaskType) {
        let thread_key = 2u32.pow(thread);
        loop {
            let join_attempt = self.0.fetch_or(thread_key, Ordering::SeqCst);
            if join_attempt < BitMask::WRITE_FLAG {
                return;
            }
            self.0.fetch_xor(thread_key, Ordering::SeqCst);
            // this is just a slow operation to allow locks to go thru
            // load values shouldn't be used to determine semaphore
            // behaviour, except in slow checks
            self.0.load(Ordering::SeqCst);
        }
    }

    // no return value due to blocking
    // see Self.get_access()
    pub fn get_access_mut(&mut self, thread: BitMaskType) {
        let pend_key = BitMask::generate_pending_flag(thread);
        let request_key = pend_key | BitMask::WRITE_FLAG;
        loop {
            // assume worst case scenario
            // so state reads that there is no active reads, writes, or pendings
            let state = self.0.fetch_or(request_key, Ordering::SeqCst);
            // now a higher level thread comes and sees that the thread is being
            // written to, so it cannot access it. This is the desired behaviour
            // as level exists only to prevent deadlocks. Priority is handled by
            // a scheduler. Two threads cannot concurrently set themselves as
            // writing due to the atomicity of the underlying data
            let wait_for_turn = state > pend_key;
            let wait_for_read_end = (state | BitMask::NON_READ_FLAGS ^ BitMask::NON_READ_FLAGS) > 0;
            if !(wait_for_turn || wait_for_read_end) {
                self.0.fetch_xor(pend_key, Ordering::SeqCst);
                return;
            }
            // need to update the fact that the thread isn't actually writing
            self.0
                .fetch_and(state ^ !BitMask::LOCK_VALUE, Ordering::SeqCst);
            // slow operation to allow other threads time to do things
            self.0.load(Ordering::SeqCst);
        }
    }

    pub fn drop_read_access(&mut self, thread: BitMaskType) {
        let thread_key = 2u32.pow(thread);
        self.0.fetch_xor(thread_key, Ordering::SeqCst);
    }

    pub fn drop_write_access(&mut self) {
        self.0.fetch_xor(BitMask::LOCK_VALUE, Ordering::SeqCst);
    }
}

// Meta data relevant to an object
//
pub union Tag {
    single: u32,
    double: [u16; 2],
    quad: [u8; 4],
}

pub(crate) struct BoxMetaData {
    pub access: BitMask,
    pub tag1: Tag,
    pub tag2: Tag,
    pub tag3: Tag,
}

impl BoxMetaData {
    pub fn new() -> Self {
        Self {
            access: BitMask::new(0),
            tag1: Tag { single: 0 },
            tag2: Tag { single: 0 },
            tag3: Tag { single: 0 },
        }
    }

    pub fn get_access_ptr(&mut self) -> *mut BitMask {
        &raw mut self.access
    }
}
