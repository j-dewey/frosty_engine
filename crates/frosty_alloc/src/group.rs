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

use crate::{AllocId, FrostyAllocatable, ObjectHandleMut};

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

pub struct AllocGroup {
    pub(crate) header: Header,
    pub(crate) objs: Vec<(AllocId, Box<[u8]>)>,
}

impl AllocGroup {
    pub fn new() -> Self {
        Self {
            header: Header::new(),
            objs: Vec::new(),
        }
    }

    // An object pushed into a group is unreachable except via SharedResource<T>
    // and Query<T>
    pub fn push_obj<T: FrostyAllocatable + 'static>(&mut self, obj: T) {
        let ptr = &obj as *const T as *const u8;
        let as_bytes = unsafe { std::slice::from_raw_parts(ptr, std::mem::size_of::<T>()) };
        let boxed_data = as_bytes.to_vec().into_boxed_slice();
        self.objs.push((T::id(), boxed_data));
    }
}
