use crate::frosty_box::FrostyBox;
use crate::FrostyAllocatable;

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
