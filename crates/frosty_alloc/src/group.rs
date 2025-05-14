// While being able to allocate individual components is sometimes
// sufficient, there are many times where one would want to allocate
// a group of components while still allowing them to interact.
//
// AllocGroups are a solution to this problem. They are a collection of
// FrostyAllocatable objects which all get allocated at once and are
// able to reference eachother.
//
// Once an AllocGroup is pushed into the Allocator, it is dissolved into
// its header and stored objects.

use hashbrown::HashMap;

use crate::{AllocId, DataAccess, DataAccessMut, FrostyAllocatable, ObjectHandleMut};

pub trait NeedsSharedResource {
    fn shared_ids() -> Vec<AllocId>
    where
        Self: Sized;
    fn set_resources(&mut self, handles: Vec<ObjectHandleMut<u8>>);
}

// A pointer to another object stored in the same AllocGroup
pub struct SharedResource<T: FrostyAllocatable> {
    handle: ObjectHandleMut<T>,
}

impl<T: FrostyAllocatable> SharedResource<T> {
    // SAFETY:
    //      A handle cannot be properly constructed until all data is allocated
    //      into Allocator
    pub unsafe fn new() -> Self {
        Self {
            handle: ObjectHandleMut::new_uninit(),
        }
    }

    pub fn set_handle(&mut self, handle: ObjectHandleMut<T>) {
        self.handle = handle;
    }

    pub fn get_access(&mut self, thread: u32) -> Option<DataAccess<T>> {
        self.handle.get_access(thread)
    }

    pub fn get_access_mut(&mut self, thread: u32) -> Option<DataAccessMut<T>> {
        self.handle.get_access_mut(thread)
    }
}

struct Header {
    children: HashMap<AllocId, SharedResource<u8>>,
}

impl Header {
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
        }
    }
}

type HandleSetFn = Box<dyn FnOnce(ObjectHandleMut<u8>, Vec<ObjectHandleMut<u8>>)>;

pub struct AllocGroup {
    pub(crate) header: Header,
    pub(crate) objs: Vec<(AllocId, Box<[u8]>)>,
    pub(crate) handle_set_fns: Vec<(AllocId, Vec<AllocId>, HandleSetFn)>,
}

impl AllocGroup {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            objs: Vec::new(),
            handle_set_fns: Vec::new(),
        }
    }

    pub fn get_ids(&self) -> Vec<AllocId> {
        self.objs.iter().map(|(id, _)| *id).collect()
    }

    // An object pushed into a group is unreachable except via SharedResource<T>
    // and Query<T>
    pub fn push_obj<T: FrostyAllocatable + 'static>(&mut self, obj: T) {
        let ptr = &obj as *const T as *const u8;
        let as_bytes = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) };
        let boxed_data = as_bytes.to_vec().into_boxed_slice();
        self.objs.push((T::id(), boxed_data));
    }

    pub fn chain_push_obj<T: FrostyAllocatable + 'static>(mut self, obj: T) -> Self {
        let ptr = &obj as *const T as *const u8;
        let as_bytes = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) };
        let boxed_data = as_bytes.to_vec().into_boxed_slice();
        self.objs.push((T::id(), boxed_data));
        self
    }

    // An object pushed into a group is unreachable except via SharedResource<T>
    // and Query<T>
    pub fn push_handle_obj<T: FrostyAllocatable + NeedsSharedResource + 'static>(
        &mut self,
        obj: T,
    ) {
        let ptr = &obj as *const T as *const u8;
        let as_bytes = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) };
        let boxed_data = as_bytes.to_vec().into_boxed_slice();

        let handle_setter = |needs_handles: ObjectHandleMut<u8>,
                             handles: Vec<ObjectHandleMut<u8>>| {
            // shouldn't ever fail since needs_handles should have just been allocated before
            // this closure is called
            let mut casted_handle: DataAccessMut<T> = needs_handles
                .cast_clone()
                .get_access_mut(0)
                .expect("Object Handle lost during allocation of group");
            casted_handle.as_mut().set_resources(handles);
        };

        self.objs.push((T::id(), boxed_data));
        self.handle_set_fns
            .push((T::id(), T::shared_ids(), Box::new(handle_setter)));
    }

    pub fn chain_push_handle_obj<T: FrostyAllocatable + NeedsSharedResource + 'static>(
        mut self,
        obj: T,
    ) -> Self {
        let ptr = &obj as *const T as *const u8;
        let as_bytes = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) };
        let boxed_data = as_bytes.to_vec().into_boxed_slice();

        let handle_setter = |needs_handles: ObjectHandleMut<u8>,
                             handles: Vec<ObjectHandleMut<u8>>| {
            // shouldn't ever fail since needs_handles should have just been allocated before
            // this closure is called
            let mut casted_handle: DataAccessMut<T> = needs_handles
                .cast_clone()
                .get_access_mut(0)
                .expect("Object Handle lost during allocation of group");
            casted_handle.as_mut().set_resources(handles);
        };

        self.objs.push((T::id(), boxed_data));
        self.handle_set_fns
            .push((T::id(), T::shared_ids(), Box::new(handle_setter)));
        self
    }
}
