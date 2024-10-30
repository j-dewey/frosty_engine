use std::ops::Deref;

use frosty_alloc::{DataAccessMut, FrostyAllocatable, ObjectHandleMut};

pub(crate) enum QueryForm {
    // take all objects at once
    Continuous,
    // only take one object at a time
    Discrete(u8),
}

pub(crate) struct Query<'a, T>
where
    T: FrostyAllocatable,
{
    objs: Vec<ObjectHandleMut<T>>,
    raw: &'a mut RawQuery,
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
}
