use crate::FrostyAllocatable;

pub(crate) struct BITMASK(u8);

pub(crate) struct FrostyBox<T: FrostyAllocatable> {
    semaphore: BITMASK,
    data: T,
}
