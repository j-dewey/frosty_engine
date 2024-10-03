use std::ptr::NonNull;

use crate::{frosty_box::FrostyBox, FrostyAllocatable, ObjectHandle, ObjectHandleMut};

pub(crate) struct InterimPtr {
    pub(crate) freed: bool,
    pub(crate) active_handles: u32,
    // data pointer: quick access during gameloop
    // index:        slow access during allocator resize
    pub(crate) data: NonNull<u8>,
    pub(crate) index: usize,
}

impl InterimPtr {
    pub unsafe fn new<T: FrostyAllocatable>(data: &mut FrostyBox<T>, index: usize) -> Self {
        Self {
            freed: false,
            active_handles: 0,
            data: NonNull::new_unchecked(data as *mut FrostyBox<T> as *mut u8),
            index,
        }
    }

    pub(crate) fn free(&mut self) {
        self.freed = true;
    }

    // Returns a clone of internal ptr to FrostyBox<T> if the data
    // has not been free'd. If it has been, returns None
    pub(crate) fn try_clone_ptr<T: FrostyAllocatable>(&self) -> Option<NonNull<FrostyBox<T>>> {
        if self.freed {
            return None;
        }
        Some(self.data.clone().cast())
    }

    // Returns a clone of internal ptr to FrostyBox<T> without checking
    // if it has been free'd. Useful for accessing data which has the same
    // lifetime as the Scene
    pub(crate) unsafe fn clone_ptr_unchecked<T: FrostyAllocatable>(&self) -> NonNull<FrostyBox<T>> {
        self.data.clone().cast()
    }

    pub fn get_handle<T: FrostyAllocatable>(&self) -> ObjectHandle<T> {
        todo!()
    }

    pub fn get_handle_mut<T: FrostyAllocatable>(&mut self) -> ObjectHandleMut<T> {
        todo!()
    }

    pub fn get_ref<T: FrostyAllocatable>(&self) -> Option<&T> {
        todo!()
    }

    pub fn get_mut<T: FrostyAllocatable>(&mut self) -> Option<&mut T> {
        todo!()
    }
}
