use std::marker::{PhantomData, Unsize};

use frosty_alloc::{DataAccessMut, DynObjectHandle, FrostyAllocatable, ObjectHandleMut};

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
    pub(crate) thread: u32,
    _pd: PhantomData<T>,
}

impl<T> Query<T>
where
    T: FrostyAllocatable,
{
    pub(crate) fn new(raw: &RawQuery, thread_id: u32) -> Self {
        Self {
            raw: raw as *const RawQuery as *mut RawQuery,
            obj_ptr: 0,
            thread: thread_id,
            _pd: PhantomData,
        }
    }

    pub unsafe fn cast<U: FrostyAllocatable>(self) -> Query<U> {
        Query {
            raw: self.raw,
            obj_ptr: self.obj_ptr,
            thread: self.thread,
            _pd: PhantomData,
        }
    }
}

// is this safe?
//      obj_ptr
//          owned by thread, so safe. If iterations are spread across threads, then
//          this should become atomic
//      raw
//
unsafe impl<T: FrostyAllocatable + Send> Send for Query<T> {}

impl<'a, T> Iterator for &'a mut Query<T>
where
    T: FrostyAllocatable,
{
    type Item = DataAccessMut<T>;
    // this has a &mut &mut Query<T> parameter
    // but this is required to maintain the lifetime
    // bound on Item
    fn next(&mut self) -> Option<Self::Item> {
        let inner_array = unsafe {
            &mut self
                .raw
                .as_mut()
                .expect("Failed to read from raw query")
                .objs
        };
        if self.obj_ptr == inner_array.len() {
            return None;
        }
        let handle = unsafe { inner_array.get_unchecked_mut(self.obj_ptr) };
        self.obj_ptr += 1;
        Some(
            handle
                .cast_clone()
                .get_access_mut(self.thread)
                .expect("Failed to access component data"),
        )
    }
}

impl<T: FrostyAllocatable> Query<T> {
    pub fn next(&mut self, thread: u32) -> Option<DataAccessMut<T>> {
        let inner_array = unsafe {
            &mut self
                .raw
                .as_mut()
                .expect("Failed to read from raw query")
                .objs
        };
        if self.obj_ptr == inner_array.len() {
            return None;
        }
        let handle = unsafe { inner_array.get_unchecked_mut(self.obj_ptr) };
        self.obj_ptr += 1;
        Some(
            handle
                .cast_clone()
                .get_access_mut(self.thread)
                .expect("Failed to access component data"),
        )
    }

    pub fn next_handle(&mut self) -> Option<ObjectHandleMut<T>> {
        let objs = &mut unsafe { self.raw.as_mut() }.unwrap().objs;
        let next = objs.get_mut(self.obj_ptr)?;
        self.obj_ptr += 1;
        Some(next.cast_clone())
    }

    // resets iteration
    pub fn reset(&mut self) {
        self.obj_ptr = 0;
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

    // SAFETY
    //      Accesses object handles directly, so need to make sure
    //      RawQuery isn't destroyed
    // Returns none if self.raw fails to return a ref
    pub unsafe fn as_slice<'a>(self) -> Option<&'a [ObjectHandleMut<u8>]> {
        Some(&self.raw.as_ref()?.objs[..])
    }

    // A debug method that prints out an identifiable number
    pub fn print_id(&self) {
        println!("[QUERY ID]: {:p}", self.raw);
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

    // resets counter
    pub fn reset(&mut self) {
        self.obj_ptr = 0;
    }

    pub fn get_count(&self) -> usize {
        self.objs.len()
    }

    pub fn next(&mut self) -> Option<&mut DynObjectHandle<T>> {
        let next = self.objs.get_mut(self.obj_ptr)?;
        self.obj_ptr += 1;
        Some(next)
    }
}

impl<T> Iterator for &mut DynQuery<T>
where
    T: FrostyAllocatable + ?Sized,
{
    type Item = DynObjectHandle<T>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.objs.get_mut(self.obj_ptr)?;
        self.obj_ptr += 1;
        Some(next.clone())
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

    use frosty_alloc::{Allocator, FrostyAllocatable};

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

    unsafe impl FrostyAllocatable for Dummy {}

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
            thread: 0,
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
