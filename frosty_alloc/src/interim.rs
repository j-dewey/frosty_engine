use std::ptr::NonNull;

use crate::{frosty_box::FrostyBox, FrostyAllocatable, ObjectHandle, ObjectHandleMut};

pub(crate) struct InterimPtr<T: FrostyAllocatable> {
    freed: u8,
    active_handles: u32,
    data: NonNull<FrostyBox<T>>,
}

impl<T> InterimPtr<T>
where
    T: FrostyAllocatable,
{
    pub unsafe fn new(data: &mut FrostyBox<T>) -> Self {
        Self {
            freed: 0,
            active_handles: 0,
            data: NonNull::new_unchecked(data as *mut FrostyBox<T>),
        }
    }

    pub fn free(&mut self) {
        self.freed = 1;
    }

    pub fn get_handle(&self) -> ObjectHandle<T> {
        todo!()
    }

    pub fn get_handle_mut(&mut self) -> ObjectHandleMut<T> {
        todo!()
    }
}
