use std::{marker::PhantomData, ptr::NonNull};

use crate::{frosty_box::FrostyBox, interim::InterimPtr, FrostyAllocatable};

/*  What is up with all the pointers?
 *      1) FrostyBox<T>
 *      2) InterimPtr<T>
 *      3) ObjectHandle(Mut)<T>
 *      4) DataAccess(Mut)<T>
 *
 *  FrostyBox<T>
 *      This isn't really a pointer, but a bundling of some [T]
 *      with a semaphore to control access. This is the baseline
 *      used by all other pointers even though they use a ref
 *      and mut interface.
 *
 *  InterimPtr<T>
 *      This exists between the [FrostyBox<T>] and [ObjectHandle<T>].
 *      Since [Query]s exist outside the scope of [Allocator], there
 *      needs to be some way for information about a [FrostyBox<T>]
 *      to exist after the data has been free'd. That is what
 *      [InterimPtr<T>] is for.
 *
 *  ObjectHandle<T>
 *      This is a ptr held by a [Query] or a {Component} to access
 *      some [FrostyBox<T>]
 *
 *  DataAccess<T>
 *      This is a nice ptr interface that automatically locks and
 *      unlocks a [FrostyBox<T>] as it enters and leaves scope.
 *      It is returned by ObjectHandle<T> and should not be stored
 *      in a {Component}
 *
 *
 *
 *      ------------------------------
 *      | System    --------------   |
 *      |           | DataAccess |   |
 *      |           --------------   |
 *      ----------------|-------------
 *                      |
 *              --------|----------------------
 *              | Query |                     |
 *              |       |  ----------------   |
 *              |       |  | ObjectHandle |   |
 *              |       |  |--------------|   |
 *              |       -> | ObjectHandle |-| |
 *              |          ---------------- | |
 *              ----------------------------|--
 *                                          |
 *                        ------------------|--------------------------------
 *                        | Allocator       |                               |
 *                        |                 V                               |
 *                        |           --------------                        |
 *                        |           | InterimPtr |                        |
 *                        |           --------------                        |
 *                        |                  |                              |
 *                        |                  V                              |
 *                        |   -------------------------------------------   |
 *                        |   | FrostyBox | FrostyBox | FrostyBox | ... |   |
 *                        |   -------------------------------------------   |
 *`                       ---------------------------------------------------
 */

//
//      Data Access
//

pub struct DataAccess<T: FrostyAllocatable> {
    ptr: NonNull<FrostyBox<T>>,
    thread: u32,
}

impl<T: FrostyAllocatable> DataAccess<T> {
    pub unsafe fn cast<U: FrostyAllocatable>(&self) -> DataAccess<U> {
        DataAccess {
            ptr: self.ptr.clone().cast(),
            thread: self.thread,
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref().get_ref() }
    }
}

// Need to override drop to make sure read access is
// returned to [FrostyBox]
impl<T: FrostyAllocatable> Drop for DataAccess<T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.as_mut().drop_read_access(self.thread);
        }
    }
}

//
//      DataAccessMut
//

pub struct DataAccessMut<T: FrostyAllocatable> {
    ptr: NonNull<FrostyBox<T>>,
    thread: u32,
}

impl<T: FrostyAllocatable> DataAccessMut<T> {
    pub unsafe fn cast<U: FrostyAllocatable>(&self) -> DataAccessMut<U> {
        DataAccessMut {
            ptr: self.ptr.clone().cast(),
            thread: self.thread,
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.ptr.as_ref().get_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.ptr.as_mut().get_mut() }
    }

    pub fn drop_mut(self) -> DataAccess<T> {
        // dropping (self) will remove write access,
        // but for [DataAccess] to be safe we need it to have
        // read access before return. Moving (self) into the
        // closure will drop the write access but allow us to
        // handle the ptr and thread data before returning from
        // method
        let (mut ptr, thread) = (move |v: Self| (v.ptr, v.thread))(self);
        unsafe { ptr.as_mut().get_access(thread) };
        DataAccess { ptr, thread }
    }
}

// Need to override drop to make sure read access is
// returned to [FrostyBox]
impl<T: FrostyAllocatable> Drop for DataAccessMut<T> {
    fn drop(&mut self) {
        unsafe {
            self.ptr.as_mut().drop_write_access();
        }
    }
}

//
//      ObjectHandle
//

pub struct ObjectHandle<T: FrostyAllocatable> {
    pub(crate) ptr: NonNull<InterimPtr>,
    pub(crate) _pd: PhantomData<T>,
}

impl<T: FrostyAllocatable> ObjectHandle<T> {
    pub(crate) fn get_mut(&mut self) -> &mut InterimPtr {
        unsafe { self.ptr.as_mut() }
    }

    pub fn get_access(&mut self, thread: u32) -> Option<DataAccess<T>> {
        let ptr = unsafe {
            let mut p = self.ptr.as_ref().try_clone_ptr()?;
            p.as_mut().get_access(thread);
            p
        };
        Some(DataAccess { ptr, thread })
    }
}

// These are safe since data is only accessible through a DataAccesss
unsafe impl<T: FrostyAllocatable> Sync for ObjectHandle<T> {}
unsafe impl<T: FrostyAllocatable> Send for ObjectHandle<T> {}

//
//      ObjectHandleMut
//

pub struct ObjectHandleMut<T: FrostyAllocatable> {
    pub(crate) ptr: NonNull<InterimPtr>,
    pub(crate) _pd: PhantomData<T>,
}

impl<T: FrostyAllocatable> ObjectHandleMut<T> {
    pub fn get_access(&mut self, thread: u32) -> Option<DataAccess<T>> {
        let ptr = unsafe {
            let mut p = self.ptr.as_ref().try_clone_ptr()?;
            p.as_mut().get_access(thread);
            p
        };
        Some(DataAccess { ptr, thread })
    }

    pub fn get_access_mut(&mut self, thread: u32) -> Option<DataAccessMut<T>> {
        let ptr = unsafe {
            let mut p = self.ptr.as_ref().try_clone_ptr()?;
            p.as_mut().get_access(thread);
            p
        };
        Some(DataAccessMut { ptr, thread })
    }

    pub unsafe fn dissolve_data(&mut self) -> ObjectHandleMut<u8> {
        ObjectHandleMut {
            ptr: self.ptr,
            _pd: PhantomData,
        }
    }
}

unsafe impl<T: FrostyAllocatable> Sync for ObjectHandleMut<T> {}
unsafe impl<T: FrostyAllocatable> Send for ObjectHandleMut<T> {}
