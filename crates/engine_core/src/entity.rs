use std::any::TypeId;

use frosty_alloc::{AllocId, FrostyAllocatable, ObjectHandle};
use hashbrown::HashMap;

type ComponentLocations = HashMap<TypeId, usize>;

// A trait that indicates one component
// references another component owned by
// the same Entity
pub trait ReferencesSiblingComponent: FrostyAllocatable {
    // The AllocId's of the references objects
    const REFERENCE_IDS: &'static [AllocId];
    // Update references based on new location of
    // all components stored by Entity
    fn update_references(&mut self, locs: &ComponentLocations);
}

// A reference to a component stored in the same
// entity. This is how a component object should
pub struct SiblingComponent<T: FrostyAllocatable> {
    handle: Option<ObjectHandle<T>>,
}

impl<T: FrostyAllocatable> SiblingComponent<T> {
    pub fn new() -> Self {
        todo!()
    }
}

// An entity is essentially an object
// composed of components and is the linchpin
// of the ECS.
// Components in an Entity can reference and
// update eachother. Components can be added
// to an Entity at any time up to when it is
// added to the Allocator
pub struct Entity {
    locations: ComponentLocations,
    comps: Vec<Box<dyn FrostyAllocatable>>,
}

impl Entity {
    pub fn new() -> Self {
        Self {
            locations: HashMap::new(),
            comps: Vec::new(),
        }
    }

    // Add a component to this entity.
    // If T: ReferencesSiblingComponent, use
    // add_with_sibling instead
    pub fn add<T>(&mut self, comp: T)
    where
        T: FrostyAllocatable,
    {
        self.locations.insert(T::id(), self.comps.len());
        self.comps.push(Box::new(comp));
    }

    // Add a component which references other components
    // stored in the same Entity.
    pub fn add_with_siblings<T>(&mut self, comp: T)
    where
        T: ReferencesSiblingComponent,
    {
        todo!()
    }

    // Drop entity while returning components
    pub(crate) fn dissolve(self) -> (ComponentLocations, Vec<Box<dyn FrostyAllocatable>>) {
        (self.locations, self.comps)
    }
}
