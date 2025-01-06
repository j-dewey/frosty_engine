use std::{
    marker::{PhantomData, Unsize},
    ptr::NonNull,
};

use crate::{frosty_box::BitMask, interim::InterimPtr, FrostyAllocatable};

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

pub struct DataAccess<T: FrostyAllocatable + ?Sized> {
    data: NonNull<T>,
    access: NonNull<BitMask>,
    thread: u32,
}

impl<T: FrostyAllocatable> DataAccess<T> {
    pub unsafe fn cast<U: FrostyAllocatable>(self) -> DataAccess<U> {
        DataAccess {
            data: self.data.clone().cast(),
            access: self.access.clone(),
            thread: self.thread,
        }
    }

    pub unsafe fn cast_dyn<U: FrostyAllocatable + ?Sized>(self) -> DataAccess<U>
    where
        T: Unsize<U>,
    {
        let data_ptr = self.data.as_ptr();
        DataAccess {
            data: NonNull::new(data_ptr as *mut U).unwrap(),
            access: self.access.clone(),
            thread: self.thread,
        }
    }
}

impl<T: FrostyAllocatable + ?Sized> DataAccess<T> {
    pub fn as_ref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }
}

// Need to override drop to make sure read access is
// returned to [FrostyBox]
impl<T: FrostyAllocatable + ?Sized> Drop for DataAccess<T> {
    fn drop(&mut self) {
        unsafe {
            self.access.as_mut().drop_read_access(self.thread);
        }
    }
}

//
//      DataAccessMut
//

pub struct DataAccessMut<T: FrostyAllocatable + ?Sized> {
    data: NonNull<T>,
    access: NonNull<BitMask>,
    thread: u32,
}

impl<T: FrostyAllocatable + ?Sized> DataAccessMut<T> {
    pub unsafe fn cast<U: FrostyAllocatable>(&self) -> DataAccessMut<U> {
        DataAccessMut {
            data: self.data.clone().cast(),
            access: self.access.clone(),
            thread: self.thread,
        }
    }

    pub fn as_ref(&self) -> &T {
        unsafe { self.data.as_ref() }
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe { self.data.as_mut() }
    }

    pub fn drop_mut(self) -> DataAccess<T> {
        // dropping (self) will remove write access,
        // but for [DataAccess] to be safe we need it to have
        // read access before return. Moving (self) into the
        // closure will drop the write access but allow us to
        // handle the ptr and thread data before returning from
        // method
        let (data, mut access, thread) = (move |v: Self| (v.data, v.access, v.thread))(self);
        unsafe { access.as_mut().get_access(thread) };
        DataAccess {
            data,
            access,
            thread,
        }
    }
}

// Need to override drop to make sure read access is
// returned to [FrostyBox]
impl<T: FrostyAllocatable + ?Sized> Drop for DataAccessMut<T> {
    fn drop(&mut self) {
        unsafe {
            self.access.as_mut().drop_write_access();
        }
    }
}

//
//      ObjectHandle
//

pub struct ObjectHandle<T: FrostyAllocatable + ?Sized> {
    pub(crate) ptr: NonNull<InterimPtr>,
    pub(crate) _pd: PhantomData<T>,
}

impl<T: FrostyAllocatable> ObjectHandle<T> {
    pub(crate) fn get_mut(&mut self) -> &mut InterimPtr {
        unsafe { self.ptr.as_mut() }
    }

    pub fn get_access(&mut self, thread: u32) -> Option<DataAccess<T>> {
        let (data_ptr, access_ptr) = unsafe {
            let p = self.ptr.as_ref().try_clone_ptr()?.as_mut();
            p.get_access(thread);
            p.get_ptrs()
        };
        Some(DataAccess {
            data: NonNull::new(data_ptr).unwrap(),
            access: NonNull::new(access_ptr).unwrap(),
            thread,
        })
    }
}

// These are safe since data is only accessible through a DataAccesss
unsafe impl<T: FrostyAllocatable> Sync for ObjectHandle<T> {}
unsafe impl<T: FrostyAllocatable> Send for ObjectHandle<T> {}

//
//      ObjectHandleMut
//

pub struct ObjectHandleMut<T: FrostyAllocatable + ?Sized> {
    pub(crate) ptr: NonNull<InterimPtr>,
    pub(crate) _pd: PhantomData<T>,
}

impl<T: FrostyAllocatable> ObjectHandleMut<T> {
    pub fn get_access(&mut self, thread: u32) -> Option<DataAccess<T>> {
        let (data_ptr, access_ptr) = unsafe {
            let p = self.ptr.as_ref().try_clone_ptr()?.as_mut();
            p.get_access(thread);
            p.get_ptrs()
        };
        Some(DataAccess {
            data: NonNull::new(data_ptr).unwrap(),
            access: NonNull::new(access_ptr).unwrap(),
            thread,
        })
    }

    pub fn get_access_mut(&mut self, thread: u32) -> Option<DataAccessMut<T>> {
        let (data_ptr, access_ptr) = unsafe {
            let p = self.ptr.as_ref().try_clone_ptr()?.as_mut();
            p.get_access(thread);
            p.get_ptrs()
        };
        Some(DataAccessMut {
            data: NonNull::new(data_ptr).unwrap(),
            access: NonNull::new(access_ptr).unwrap(),
            thread,
        })
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

// An object handle which stores trait objects
pub struct DynObjectHandle<T: FrostyAllocatable + ?Sized> {
    data: NonNull<T>,
    access: NonNull<BitMask>,
}

impl<T: FrostyAllocatable + ?Sized> DynObjectHandle<T> {
    pub fn new<U: FrostyAllocatable>(handle: ObjectHandleMut<U>) -> Self
    where
        U: Unsize<T>,
    {
        let (data_ptr, access_ptr): (*mut U, *mut BitMask) = unsafe {
            handle
                .ptr
                .as_ref()
                .try_clone_ptr()
                .unwrap()
                .as_mut()
                .get_ptrs()
        };
        Self {
            data: NonNull::new(data_ptr).unwrap(),
            access: NonNull::new(access_ptr).unwrap(),
        }
    }

    pub fn get_access(&mut self, thread: u32) -> Option<DataAccess<T>> {
        unsafe {
            self.access.as_mut().get_access(thread);
        }
        Some(DataAccess {
            data: self.data.clone(),
            access: self.access.clone(),
            thread,
        })
    }

    pub fn get_access_mut(&mut self, thread: u32) -> Option<DataAccessMut<T>> {
        unsafe {
            self.access.as_mut().get_access_mut(thread);
        }
        Some(DataAccessMut {
            data: self.data.clone(),
            access: self.access.clone(),
            thread,
        })
    }
}
