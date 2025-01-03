use std::marker::PhantomData;

use frosty_alloc::{DataAccess, DataAccessMut, FrostyAllocatable, ObjectHandleMut};

#[derive(Clone)]
pub(crate) enum QueryForm {
    // take all objects at once
    Continuous,
    // only take one object at a time
    Discrete(u8),
}

#[derive(Copy, Clone)]
pub struct Query<T>
where
    T: FrostyAllocatable + ?Sized,
{
    raw: *mut RawQuery,
    obj_ptr: usize,
    _pd: PhantomData<T>,
}

// is this safe?
//      obj_ptr
//          owned by thread, so safe. If iterations are spread across threads, then
//          this should become atomic
//      raw
//
unsafe impl<T: FrostyAllocatable + Send> Send for Query<T> {}

impl<T: FrostyAllocatable> Query<T> {
    pub fn next(&mut self, thread: u32) -> Option<DataAccess<T>> {
        let objs = &mut unsafe { self.raw.as_mut() }.unwrap().objs;
        let next = objs.get_mut(self.obj_ptr)?;
        self.obj_ptr += 1;
        unsafe { Some(next.get_access(thread)?.cast()) }
    }
}

pub(crate) struct RawQuery {
    form: QueryForm,
    objs: Vec<ObjectHandleMut<u8>>,
    to_drop: Vec<usize>,
    indx: usize,
}

impl RawQuery {
    pub fn new(form: QueryForm, objs: Vec<ObjectHandleMut<u8>>) -> Self {
        Self {
            form,
            objs,
            to_drop: Vec::new(),
            indx: 0,
        }
    }

    pub fn next(&mut self, thread: u32) -> Option<DataAccessMut<u8>> {
        match self.objs.get_mut(self.indx).unwrap().get_access_mut(thread) {
            // data was free'd
            None => {
                self.to_drop.push(self.indx);
                self.indx += 1;
                None
            }
            Some(ptr) => {
                self.indx += 1;
                Some(ptr)
            }
        }
    }

    pub(crate) fn add_handle(&mut self, handle: ObjectHandleMut<u8>) {
        self.objs.push(handle);
    }
}
