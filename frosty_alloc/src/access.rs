use crate::frosty_box::FrostyBox;
use crate::FrostyAllocatable;

use std::cell::{RefCell, UnsafeCell};
use std::ptr::NonNull;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AllocName {
    uoid: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AllocId {
    uid: u64,
}

impl AllocId {
    pub fn new(val: u64) -> Self {
        Self { uid: val }
    }
}

// An [ObjectHandle<T>] and a [ObjectHandleMut<T>] are both
// interfaces that allow threads to safely interact with
// [FrostyBox<T>]s stored in the [Allocator]. The underlying
// data stored in each handle is the same, but the mut is
// used for code distinction
pub struct ObjectHandle<T: FrostyAllocatable> {
    ptr: NonNull<FrostyBox<T>>,
}

impl<T: FrostyAllocatable> ObjectHandle<T> {
    pub fn as_ref(&mut self, thread: u32) -> &T {
        let ptr = unsafe { self.ptr.as_mut() };
        ptr.get_access(thread);
        ptr.get_ref()
    }
}

pub struct ObjectHandleMut<T: FrostyAllocatable> {
    ptr: NonNull<FrostyBox<T>>,
}

impl<T: FrostyAllocatable> ObjectHandleMut<T> {
    pub fn as_ref(&mut self, thread: u32) -> &T {
        let ptr = unsafe { self.ptr.as_mut() };
        ptr.get_access(thread);
        ptr.get_ref()
    }

    pub fn as_mut(&mut self, thread: u32) -> &mut T {
        let ptr = unsafe { self.ptr.as_mut() };
        ptr.get_access_mut(thread);
        ptr.get_mut()
    }
}
