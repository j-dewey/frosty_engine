use crate::{BitMask, BitMaskType, BoxMetaData, FrostyAllocatable, Tag};
use std::{
    fs::Metadata,
    mem::MaybeUninit,
    sync::atomic::{AtomicU32, Ordering},
    u32,
};

// This represents some item stored in [Allocator] with a semaphore to
// allow for multi-thread reading. This is not a pointer and cannot be
// shared across threads, but acts as an intermediary between [ObjectHandle<T>]
// and the actual [Allocator]
pub(crate) struct FrostyBox<T: FrostyAllocatable + ?Sized> {
    pub(crate) meta: BoxMetaData,
    data: T,
}

impl<T: FrostyAllocatable> FrostyBox<T> {
    pub fn new(data: T) -> Self {
        Self {
            meta: BoxMetaData::new(),
            data,
        }
    }

    // SAFETY:
    //      This moves the resources out of obj, but leaves obj
    //      around. If this is bad, need to forget obj without calling
    //      destructor
    pub unsafe fn from_raw(obj: *mut T) -> Self {
        //  Create box with all zeroed data
        //  swap zeroed data with obj data
        //  return box
        unsafe {
            let mut partial_init: Self = MaybeUninit::zeroed().assume_init();
            partial_init.meta = BoxMetaData::new();
            let new = &raw mut partial_init.data;
            std::ptr::swap(obj, new);
            partial_init
        }
    }
}

impl<T: FrostyAllocatable + ?Sized> FrostyBox<T> {
    // no return value. since this method is blocking,
    // code execution begins again once access is granted
    pub fn get_access(&mut self, thread: BitMaskType) {
        self.meta.access.get_access(thread);
    }

    // no return value due to blocking
    // see Self.get_access()
    pub fn get_access_mut(&mut self, thread: BitMaskType) {
        self.meta.access.get_access_mut(thread);
    }

    pub fn drop_read_access(&mut self, thread: BitMaskType) {
        self.meta.access.drop_read_access(thread);
    }

    pub fn drop_write_access(&mut self) {
        self.meta.access.drop_write_access();
    }

    pub fn get_ref(&self) -> &T {
        &self.data
    }

    pub fn get_mut(&mut self) -> &mut T {
        &mut self.data
    }

    pub fn get_raw(&mut self) -> *mut T {
        &mut self.data as *mut T
    }

    // SAFETY:
    //    The caller has to keep track of each pointer on their own
    //    and ensure that they don't do anything bad
    pub unsafe fn get_ptrs(&mut self) -> (*mut T, *mut BoxMetaData) {
        (&raw mut self.data, &raw mut self.meta)
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
