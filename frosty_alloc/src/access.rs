use crate::FrostyAllocatable;

use std::cell::RefCell;
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};

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

pub struct ObjectHandle<T: FrostyAllocatable> {
    ptr: Arc<T>,
}

impl<T: FrostyAllocatable> ObjectHandle<T> {
    pub fn as_ref(&self) -> &T {
        self.ptr.as_ref()
    }
}

pub struct ObjectHandleMut<T: FrostyAllocatable> {
    ptr: Arc<Mutex<T>>,
}

impl<T: FrostyAllocatable> ObjectHandleMut<T> {
    pub fn as_ref(&self) -> Result<MutexGuard<T>, PoisonError<MutexGuard<T>>> {
        self.ptr.lock()
    }
}
