use crate::FrostyAllocatable;

use std::cell::RefCell;
use std::rc::Rc;

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
    ptr: Rc<T>,
}

pub struct ObjectHandleMut<T: FrostyAllocatable> {
    ptr: Rc<RefCell<T>>,
}
