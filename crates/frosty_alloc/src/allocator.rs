use std::{io::Write, marker::PhantomData, ptr::NonNull};

use hashbrown::HashMap;

use crate::{
    debug::{DebugData, DebugOutter},
    frosty_box::FrostyBox,
    group::AllocGroup,
    interim::InterimPtr,
    AllocId, FrostyAllocatable, ObjectHandle, ObjectHandleMut,
};

// Alliases
pub type Index = usize;
type HeapAlloc<T> = Box<T>;

fn repoint_interim<T: FrostyAllocatable>(
    id: AllocId,
    data: &mut Vec<FrostyBox<T>>,
    interim: &mut [Box<InterimPtr>],
) {
    let mut encountered = 0;
    for ptr in interim {
        if ptr.type_id != id {
            continue;
        }

        ptr.data = NonNull::new(&mut data[encountered] as *mut FrostyBox<T> as *mut u8)
            .expect("Failed to init new InterimPtr during interim re-pointing");
        encountered += 1;
    }
}

// A `Vec<C> where C: Component` with C type dissolved
// Using Vec allows us to take advantage of compiler and
// stdlib improvements. it also takes care of SIMD, alignment,
// and other optimizations for free.
struct DissolvedVec {
    data: Vec<u8>,
    len: usize, // size of T in bytes
}

impl DissolvedVec {
    // SAFETY:
    //      Creates a Vec<C>, but internally refers to it as a
    //      Vec<u8>. The original C type must be remembered some way
    //      so that only the proper object type is pushed
    pub unsafe fn new<C>() -> Self {
        let data: Vec<C> = Vec::new();
        Self {
            // the initial allocation referenced the C type, so this
            // vec stores the proper alignment
            data: unsafe { std::mem::transmute(data) },
            len: std::mem::size_of::<C>(),
        }
    }

    // SAFETY:
    //      Must only case to the C which self was defined with
    unsafe fn as_vec_mut<C>(&mut self) -> &mut Vec<C> {
        (&mut self.data as *mut Vec<u8> as *mut Vec<C>)
            .as_mut()
            .expect("Failed to cast SystemVec to Vec<C>")
    }
}

pub struct SystemAllocator {
    // SAFETY:
    //      The AllocId maps a type to a Vec so that the proper
    //      dissolved data type is always refered to.
    data: HashMap<AllocId, DissolvedVec>,
    interim: Vec<HeapAlloc<InterimPtr>>,
}

impl SystemAllocator {
    pub fn new() -> Self {
        Self::with_capacity(4)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            data: HashMap::with_capacity(capacity),
            interim: Vec::with_capacity(capacity),
        }
    }

    // Allocate an already initialized object. Moves the object inside of self;
    // making the object only editable and viewable from the returned ObjectHandleMut
    // or a clone of it
    pub fn alloc<T: FrostyAllocatable>(&mut self, obj: T) -> ObjectHandleMut<T> {
        let boxed_obj = FrostyBox::new(obj);
        // this block cannot be moved to a seperate method due
        // to annoying reference rules
        let vec: &mut Vec<FrostyBox<T>> = unsafe {
            if self.data.contains_key(&T::id()) {
                self.data.get_mut(&T::id()).unwrap().as_vec_mut()
            } else {
                let new_vec = DissolvedVec::new::<FrostyBox<T>>();
                self.data.insert(T::id(), new_vec);
                self.data.get_mut(&T::id()).unwrap().as_vec_mut()
            }
        };

        let old_cap = vec.capacity();

        vec.push(boxed_obj);

        if vec.capacity() != old_cap {
            repoint_interim(T::id(), vec, &mut self.interim)
        }

        let last_index = vec.len() - 1;
        let data_ptr = &mut vec[last_index] as *mut FrostyBox<T>;
        let interim = InterimPtr {
            freed: false,
            active_handles: 0,
            data: NonNull::new(data_ptr as *mut u8).unwrap(),
            type_id: T::id(),
            index: 0,
        };

        let inter_index = self.interim.len();
        self.interim.push(HeapAlloc::new(interim));
        ObjectHandleMut {
            ptr: NonNull::new(self.interim[inter_index].as_mut() as *mut InterimPtr)
                .expect("Failed to init ObjectHandleMut from InterimPtr during alloc"),
            _pd: PhantomData {},
        }
    }

    // SAFETY:
    //      self takes ownership of *data, and *data is zeroed after this call
    // Returns a handle the object stored at the pointer passed in.
    pub unsafe fn alloc_raw<T: FrostyAllocatable>(&mut self, data: *mut T) -> ObjectHandleMut<u8> {
        let boxed_obj = FrostyBox::from_raw(data);

        // this block cannot be moved to a seperate method due
        // to annoying reference rules
        let vec: &mut Vec<FrostyBox<T>> = unsafe {
            if self.data.contains_key(&T::id()) {
                self.data.get_mut(&T::id()).unwrap().as_vec_mut()
            } else {
                let new_vec = DissolvedVec::new::<FrostyBox<T>>();
                self.data.insert(T::id(), new_vec);
                self.data.get_mut(&T::id()).unwrap().as_vec_mut()
            }
        };

        let old_cap = vec.capacity();

        vec.push(boxed_obj);
        let obj_ptr = vec.last_mut().unwrap();
        let (data, bits) = obj_ptr.get_ptrs();

        if vec.capacity() != old_cap {
            repoint_interim(T::id(), vec, &mut self.interim)
        }

        let last_index = vec.len() - 1;
        let data_ptr = &mut vec[last_index] as *mut FrostyBox<T>;
        let interim = InterimPtr {
            freed: false,
            active_handles: 0,
            data: NonNull::new(data_ptr as *mut u8).unwrap(),
            type_id: T::id(),
            index: 0,
        };

        let inter_index = self.interim.len();
        self.interim.push(HeapAlloc::new(interim));
        ObjectHandleMut {
            ptr: NonNull::new(self.interim[inter_index].as_mut() as *mut InterimPtr)
                .expect("Failed to init ObjectHandleMut from InterimPtr during alloc"),
            _pd: PhantomData {},
        }
    }

    // Allocate all objects in a group and connect all
    pub fn alloc_group(&mut self, mut group: AllocGroup) -> Vec<ObjectHandleMut<u8>> {
        let mut handles = Vec::with_capacity(group.objs.len());
        let mut id_to_indx = HashMap::new();
        // allocation
        for (i, (id, data, alloc)) in group.objs.drain(..).enumerate() {
            let handle = unsafe { alloc(self, data.as_ptr() as *mut u8) };
            handles.push(handle);
            id_to_indx.insert(id, i);
        }

        // set [SharedResource]s
        for (id, needed_ids, setter_fn) in group.handle_set_fns.drain(..) {
            let shared_handles: Vec<ObjectHandleMut<u8>> = needed_ids
                .iter()
                .map(|needed_id| {
                    handles
                        .get_mut(*id_to_indx.get(needed_id).unwrap())
                        .unwrap()
                        .cast_clone()
                })
                .collect();
            let obj_handle = handles.get_mut(*id_to_indx.get(&id).unwrap()).unwrap();
            (setter_fn)(obj_handle.clone(), shared_handles);
        }
        handles
    }

    pub unsafe fn get<T: FrostyAllocatable>(&mut self, index: Index) -> Option<ObjectHandle<T>> {
        let interim = self.interim.get_mut(index)?.as_mut();
        interim.active_handles += 1;
        Some(ObjectHandle {
            ptr: NonNull::new(interim as *mut InterimPtr).unwrap(),
            _pd: PhantomData {},
        })
    }

    pub fn get_mut<T: FrostyAllocatable>(&mut self, index: Index) -> Option<ObjectHandleMut<T>> {
        let interim = self.interim.get_mut(index)?.as_mut();
        interim.active_handles += 1;
        Some(ObjectHandleMut {
            ptr: NonNull::new(interim as *mut InterimPtr).unwrap(),
            _pd: PhantomData {},
        })
    }
}

impl DebugOutter for Allocator {
    fn dump_data(&self, fs: &mut std::fs::File) {
        // --------------------------------------
        // Allocator
        //      Region id: {}
        //          Region Start
        //          Data: [
        //              000000
        //          ]
        //

        fs.write_all(b"---------------------------------------------\n")
            .unwrap();
        fs.write_all(b"Allocator\n").unwrap();

        for (id, vec) in self.data.iter() {
            let obj_width = vec.len;
            let vec_ptr = vec.data.as_ptr();
            fs.write_all(
                format!(
                    "\tRegion ID: {:?}\n \t\tRegion Start: {:?}\n \t\tData: [\n",
                    id, vec_ptr
                )
                .as_bytes(),
            )
            .unwrap();

            let mut str = String::with_capacity(obj_width + obj_width / 8 + 3);
            for i in 0..vec.data.len() {
                str.clear();
                str += format!("\t\t\t {:?} ", unsafe { vec_ptr.add(i * obj_width) }).as_str();
                for offset in 0..obj_width {
                    str += &format!("{:x} ", unsafe { *vec_ptr.add(i * obj_width + offset) })[..];
                }
                str.push('\n');

                fs.write_all(str.as_bytes()).unwrap();
            }
            fs.write_all("\t\t]\n".as_bytes()).unwrap();
        }
        fs.write_all("\tInterim: [\n".as_bytes()).unwrap();
        for int in &self.interim {
            fs.write_all(
                format!(
                    "\t\t {:?}, {:?} <freed: {:?}, active_handles: {:?}, index: {:?}, ptr: {:?}>\n",
                    int as *const Box<InterimPtr>,
                    int.as_ref() as *const InterimPtr,
                    int.freed,
                    int.active_handles,
                    int.index,
                    int.data
                )
                .as_bytes(),
            )
            .unwrap();
        }
        fs.write_all(b"\t ]\n").unwrap();
    }
}

pub type Allocator = SystemAllocator;

#[cfg(test)]
mod allocator_tests {

    use crate::FrostyAllocatable;

    use super::Allocator;

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct UniformDummy {
        a: i32,
        b: i32,
    }

    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    struct NonUniformDummy {
        a: i32,
        b: u8,
    }

    impl UniformDummy {
        pub fn to_bytes(self) -> Box<[u8]> {
            unsafe {
                let mut boxed = Box::new(self); // put in box to heap alloc
                let ptr = boxed.as_mut() as *mut Self as *mut [u8; 8];
                Box::from(&(*ptr)[..])
            }
        }
    }
    unsafe impl FrostyAllocatable for UniformDummy {}

    impl NonUniformDummy {
        pub fn to_bytes(self) -> Box<[u8]> {
            unsafe {
                let mut boxed = Box::new(self); // put in box to heap alloc
                let ptr = boxed.as_mut() as *mut Self as *mut [u8; 5];
                Box::from(&(*ptr)[..])
            }
        }
    }
    unsafe impl FrostyAllocatable for NonUniformDummy {}

    #[test]
    fn allocate_primitive() {
        let mut alloc = Allocator::with_capacity(4 * 3);
        let data1 = 16;
        let data2 = 16.0;
        let data3 = 16u32;
        let _ = alloc.alloc(data1);
        let _ = alloc.alloc(data2);
        let _ = alloc.alloc(data3);
    }

    #[test]
    fn allocate_uniform_struct() {
        let mut alloc = Allocator::with_capacity(std::mem::size_of::<UniformDummy>());
        let dummy = UniformDummy { a: 10, b: 10 };
        alloc.alloc(dummy);
    }

    #[test]
    fn allocate_nonuniform_struct() {
        let mut alloc = Allocator::with_capacity(std::mem::size_of::<NonUniformDummy>());
        let dummy = NonUniformDummy { a: 10, b: 10 };
        alloc.alloc(dummy);
    }

    #[test]
    fn access_primitive() {
        let mut alloc = Allocator::with_capacity(4 * 3);
        let data1 = 16;
        let data2 = 16u32;
        let data3 = 2.0;
        let _ = alloc.alloc(data1);
        let _ = alloc.alloc(data2);
        let _ = alloc.alloc(data3);
    }

    #[test]
    fn alloc_without_resize_new() {
        let mut alloc = Allocator::new();
        alloc.alloc(1u8);
    }

    #[test]
    fn alloc_with_resize_new() {
        let mut alloc = Allocator::new();
        alloc.alloc(1u128);
    }

    #[test]
    fn alloc_non_pod() {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        struct PointedData {
            data: u8,
        }
        struct PointsToData {
            data: Vec<PointedData>,
        }
        unsafe impl FrostyAllocatable for PointsToData {}

        let mut alloc = Allocator::new();
        let mut ptr = {
            let data = PointedData { data: 10 };
            let ptr = PointsToData { data: vec![data] };

            alloc.alloc(ptr)
        };

        (|| {
            // make new a stack frame and mess around again
            let data = PointedData { data: 100 };
            let ptr = PointsToData { data: vec![data] };
        })();

        ptr.get_access_mut(0)
            .expect("Failed to access PointsToData")
            .as_mut()
            .data[0] = PointedData { data: 5 };

        assert_eq!(
            PointedData { data: 5 },
            ptr.get_access(0)
                .expect("Failed to acces PointsToData twice")
                .as_ref()
                .data[0]
        );
    }
}
