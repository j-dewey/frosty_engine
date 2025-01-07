use std::marker::{PhantomData, Unsize};

use frosty_alloc::{
    DataAccess, DataAccessMut, DynObjectHandle, FrostyAllocatable, ObjectHandleMut,
};

#[derive(Clone)]
pub(crate) enum QueryForm {
    // take all objects at once
    Continuous,
    // only take one object at a time
    Discrete(u8),
}

// A reference to all objects of type T
// Though it doesn't implement Iter,
// data can still be cycled through
// with next()
#[derive(Copy, Clone)]
pub struct Query<T>
where
    T: FrostyAllocatable + ?Sized,
{
    raw: *mut RawQuery,
    obj_ptr: usize, // index for iterating
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
    pub fn next(&mut self, thread: u32) -> Option<DataAccessMut<T>> {
        let objs = &mut unsafe { self.raw.as_mut() }.unwrap().objs;
        let next = objs.get_mut(self.obj_ptr)?;
        self.obj_ptr += 1;
        unsafe { Some(next.get_access_mut(thread)?.cast()) }
    }

    // Consume a Query. Move the pointers stored in the
    // RawQuery into a DynQuery without removing them from the
    // RawQuery
    pub fn cast_dyn<U: FrostyAllocatable + ?Sized>(self) -> DynQuery<U>
    where
        T: Unsize<U>,
    {
        DynQuery {
            objs: unsafe {
                self.raw
                    .as_ref()
                    .unwrap()
                    .objs
                    .iter()
                    .map(|handle| DynObjectHandle::new(&handle.cast_clone::<T>()))
                    .collect()
            },
            obj_ptr: 0,
            _pd: PhantomData,
        }
    }
}

// A Query consisting of trait objects.
// The data is still stored on the Allocator
// and the trait object is stored in the ObjectHandle
pub struct DynQuery<T: FrostyAllocatable + ?Sized> {
    objs: Vec<DynObjectHandle<T>>,
    obj_ptr: usize,
    _pd: PhantomData<T>,
}

impl<T: FrostyAllocatable + ?Sized> DynQuery<T> {
    // Create a new query without any object handles
    pub fn new_empty() -> Self {
        Self {
            objs: Vec::new(),
            obj_ptr: 0,
            _pd: PhantomData,
        }
    }

    pub fn push<U: FrostyAllocatable>(&mut self, obj: &ObjectHandleMut<U>)
    where
        U: Unsize<T>,
    {
        self.objs.push(DynObjectHandle::new(obj))
    }

    pub fn next(&mut self) -> Option<&mut DynObjectHandle<T>> {
        let next = self.objs.get_mut(self.obj_ptr)?;
        self.obj_ptr += 1;
        Some(next)
    }
}

// The underlying data beneath a Query.
pub(crate) struct RawQuery {
    form: QueryForm,
    objs: Vec<ObjectHandleMut<u8>>,
    to_drop: Vec<usize>,
}

impl RawQuery {
    pub fn new(form: QueryForm, objs: Vec<ObjectHandleMut<u8>>) -> Self {
        Self {
            form,
            objs,
            to_drop: Vec::new(),
        }
    }

    pub(crate) fn add_handle(&mut self, handle: ObjectHandleMut<u8>) {
        self.objs.push(handle);
    }
}

#[cfg(test)]
mod query_tests {
    use std::marker::PhantomData;

    use frosty_alloc::{AllocId, Allocator, FrostyAllocatable};

    use super::{Query, QueryForm, RawQuery};

    trait HasData: FrostyAllocatable {
        fn get_data(&self) -> i32;
    }

    #[derive(Copy, Clone, Debug)]
    struct Dummy {
        data: i32,
    }

    impl HasData for Dummy {
        fn get_data(&self) -> i32 {
            self.data
        }
    }

    unsafe impl FrostyAllocatable for Dummy {
        fn id() -> frosty_alloc::AllocId
        where
            Self: Sized,
        {
            AllocId::new(1000000)
        }
    }

    #[test]
    fn test_dyn_reference() {
        let mut alloc = Allocator::new();
        let mut dummy_handle = alloc.alloc(Dummy { data: 3 }).unwrap();

        let mut raw_query = RawQuery::new(
            QueryForm::Continuous,
            vec![unsafe { dummy_handle.dissolve_data() }],
        );
        let query: Query<Dummy> = Query {
            raw: &mut raw_query as *mut RawQuery,
            obj_ptr: 0,
            _pd: PhantomData,
        };

        let mut dyn_query = query.cast_dyn::<dyn HasData>();
        let handle = dyn_query.next().unwrap();
        let num = handle
            .get_access(0)
            .expect("Failed to get access to dyn data")
            .as_ref()
            .get_data();
        assert_eq!(3, num);
    }
}
