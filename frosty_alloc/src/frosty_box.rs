use crate::FrostyAllocatable;
use std::{sync::atomic::AtomicU32, u32};

// with 32 bits:
//      2 are reserved for writing flags
//      15 are
type BitMaskType = u32;
pub(crate) struct BitMask(AtomicU32);

impl BitMask {
    pub const WRITE_FLAG: BitMaskType = 0b10_000000000000000_000000000000000;
    pub fn new(v: u32) -> Self {
        Self(AtomicU32::new(v))
    }
}

pub(crate) struct FrostyBox<T: FrostyAllocatable> {
    semaphore: BitMask,
    data: T,
}

#[cfg(test)]
mod box_tests {
    use super::*;

    #[test]
    fn write_flag_valid() {
        let proper_write_flag = BitMaskType::MAX / 2 + 1;
        assert_eq!(BitMask::WRITE_FLAG, proper_write_flag);
    }
}
