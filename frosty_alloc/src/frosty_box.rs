use crate::FrostyAllocatable;
use std::{
    sync::atomic::{AtomicU32, Ordering},
    u32,
};

// with 32 bits:
//      2 are reserved for writing flags
//      15 are
type BitMaskType = u32;
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
        Self::LOCK_VALUE * 2u32.pow(thread)
    }
}

// This represents some item stored in [Allocator] with a semaphore to
// allow for multi-thread reading. This is not a pointer and cannot be
// shared across threads, but acts as an intermediary between [ObjectHandle<T>]
// and the actual [Allocator]
pub(crate) struct FrostyBox<T: FrostyAllocatable> {
    semaphore: BitMask,
    data: T,
}

impl<T: FrostyAllocatable> FrostyBox<T> {
    pub fn new(data: T) -> Self {
        Self {
            semaphore: BitMask::new(0),
            data,
        }
    }

    // no return value. since this method is blocking,
    // code execution begins again once access is granted
    pub fn get_access(&mut self, thread: BitMaskType) {
        let thread_key = 2u32.pow(thread);
        loop {
            let join_attempt = self.semaphore.0.fetch_or(thread_key, Ordering::SeqCst);
            if join_attempt >= BitMask::WRITE_FLAG {
                return;
            }
            self.semaphore.0.fetch_xor(thread_key, Ordering::SeqCst);
            // this is just a slow operation to allow locks to go thru
            // load values shouldn't be used to determine semaphore
            // behaviour, except in slow checks
            self.semaphore.0.load(Ordering::SeqCst);
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
            let state = self.semaphore.0.fetch_or(request_key, Ordering::SeqCst);
            // now a higher level thread comes and sees that the thread is being
            // written to, so it cannot access it. This is the desired behaviour
            // as level exists only to prevent deadlocks. Priority is handled by
            // a scheduler. Two threads cannot concurrently set themselves as
            // writing due to the atomicity of the underlying data
            let wait_for_turn = state > pend_key;
            let wait_for_read_end = (state | BitMask::NON_READ_FLAGS ^ BitMask::NON_READ_FLAGS) > 0;
            if !(wait_for_turn || wait_for_read_end) {
                self.semaphore.0.fetch_xor(pend_key, Ordering::SeqCst);
                return;
            }
            // need to update the fact that the thread isn't actually writing
            self.semaphore
                .0
                .fetch_and(state ^ !BitMask::LOCK_VALUE, Ordering::SeqCst);
            // slow operation to allow other threads time to do things
            self.semaphore.0.load(Ordering::SeqCst);
        }
    }

    pub fn drop_read_access(&mut self, thread: BitMaskType) {
        let thread_key = 2u32.pow(thread);
        self.semaphore.0.fetch_xor(thread_key, Ordering::SeqCst);
    }

    pub fn drop_write_access(&mut self) {
        self.semaphore
            .0
            .fetch_xor(BitMask::LOCK_VALUE, Ordering::SeqCst);
    }

    pub fn get_ref(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }
}

#[cfg(test)]
mod box_tests {
    use super::*;

    #[test]
    fn write_flag_valid() {
        let proper_write_flag = BitMaskType::MAX / 2 + 1;
        assert_eq!(BitMask::WRITE_FLAG, proper_write_flag);
    }

    #[test]
    fn lock_value_valid() {
        let proper_lock_value = 2u32.pow(15);
        assert_eq!(BitMask::LOCK_VALUE, proper_lock_value);
    }

    #[test]
    fn generate_pend_flags() {
        // with 32 bits, there are [32-2]/2 pend flags
        // or 15 pend flags
        for f in 0..15 {
            let expected_key = 2u32.pow(f + 15);
            assert_eq!(expected_key, BitMask::generate_pending_flag(f));
        }
    }
}
