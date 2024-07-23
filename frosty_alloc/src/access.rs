use crate::FrostyAllocatable;

use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AllocName {
    uoid: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct AllocId {
    uoid: u64,
}

pub struct ObjectHandle<T: FrostyAllocatable> {
    ptr: Rc<T>,
}

pub struct ObjectHandleMut<T: FrostyAllocatable> {
    ptr: Rc<RefCell<T>>,
}
