use std::ptr::NonNull;

use crate::{frosty_box::FrostyBox, FrostyAllocatable, ObjectHandle, ObjectHandleMut};

pub(crate) struct InterimPtr<T: FrostyAllocatable> {
    freed: bool,
    active_handles: u32,
    data: NonNull<FrostyBox<T>>,
}

impl<T> InterimPtr<T>
where
    T: FrostyAllocatable,
{
    pub unsafe fn new(data: &mut FrostyBox<T>) -> Self {
        Self {
            freed: false,
            active_handles: 0,
            data: NonNull::new_unchecked(data as *mut FrostyBox<T>),
        }
    }

    pub(crate) fn free(&mut self) {
        self.freed = true;
    }

    // Returns a clone of internal ptr to FrostyBox<T> if the data
    // has not been free'd. If it has been, returns None
    pub(crate) fn try_clone_ptr(&self) -> Option<NonNull<FrostyBox<T>>> {
        if self.freed {
            return None;
        }
        Some(self.data.clone())
    }

    // Returns a clone of internal ptr to FrostyBox<T> without checking
    // if it has been free'd. Useful for accessing data which has the same
    // lifetime as the Scene
    pub(crate) unsafe fn clone_ptr_unchecked(&self) -> NonNull<FrostyBox<T>> {
        self.data.clone()
    }

    pub fn get_handle(&self) -> ObjectHandle<T> {
        todo!()
    }

    pub fn get_handle_mut(&mut self) -> ObjectHandleMut<T> {
        todo!()
    }

    pub fn get_ref(&self) -> Option<&T> {
        todo!()
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        todo!()
    }
}
