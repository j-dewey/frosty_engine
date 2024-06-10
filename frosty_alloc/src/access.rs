use crate::FrostyAllocatable;

use std::cell::RefCell;
use std::rc::Rc;

pub struct AllocName {
    uoid: u64,
}

pub struct ObjectHandle<T: FrostyAllocatable> {
    ptr: Rc<T>,
}

pub struct ObjectHandleMut<T: FrostyAllocatable> {
    ptr: Rc<RefCell<T>>,
}
