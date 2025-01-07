use frosty_alloc::{AllocId, Allocator, FrostyAllocatable, ObjectHandleMut};
use hashbrown::HashMap;

use crate::{
    query::{QueryForm, RawQuery},
    Entity,
};

type ConverterFn = for<'a, 'b> fn(
    &'a Box<(dyn FrostyAllocatable + 'static)>,
    &'b mut frosty_alloc::Allocator,
) -> ObjectHandleMut<u8>;
//    dyn FnMut(&Box<dyn FrostyAllocatable>, &mut Allocator) -> ObjectHandleMut<u8> + 'a;

#[derive(Debug, Clone, Copy)]
pub struct UnregisteredComponent;

pub struct Spawner {
    alloc: Allocator,
    queries: HashMap<AllocId, RawQuery>,
    registered_components: HashMap<AllocId, ConverterFn>,
}

impl Spawner {
    pub fn new() -> Self {
        Self {
            alloc: Allocator::new(),
            queries: HashMap::new(),
            registered_components: HashMap::new(),
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            alloc: Allocator::with_capacity(capacity),
            queries: HashMap::new(),
            registered_components: HashMap::new(),
        }
    }

    fn upcast_component<C: FrostyAllocatable>(
        obj: &Box<dyn FrostyAllocatable>,
        alloc: &mut Allocator,
    ) -> ObjectHandleMut<u8> {
        let ptr = obj.as_ref() as *const dyn FrostyAllocatable;
        let converted_data = ptr as *const C;
        let interim_index = alloc
            .alloc_raw(converted_data)
            .expect("Issue with allocating component in Entity");
        let mut handle = alloc
            .get_mut::<C>(interim_index)
            .expect("Allocator returned invalid index of interim ptr");
        unsafe { handle.dissolve_data() }
    }

    // Register a component so that it can be properly allocated
    // Also creates the Query for C
    pub fn register_component<C: FrostyAllocatable>(&mut self) {
        self.registered_components
            .insert(C::id(), Self::upcast_component::<C>);
        self.queries
            .insert(C::id(), RawQuery::new(QueryForm::Continuous, Vec::new()));
    }

    // Spawn an entity and move all its components into the allocator then
    // add them to Querys
    pub fn spawn(&mut self, entity: Entity) -> Result<(), UnregisteredComponent> {
        let (mut locs, comps) = entity.dissolve();
        locs.iter_mut().try_for_each(|(id, i)| {
            let converter = match self.registered_components.get_mut(id) {
                Some(f) => f,
                None => return Err(UnregisteredComponent),
            };
            let handle = (converter)(&comps[*i], &mut self.alloc);

            self.queries.get_mut(id).unwrap().add_handle(handle);

            Ok(())
        })
    }

    // Move some object into the allocator and add it to the Query
    pub fn spawn_obj<C: FrostyAllocatable>(&mut self, obj: C) -> Result<(), UnregisteredComponent> {
        let query = match self.queries.get_mut(&C::id()) {
            Some(query) => query,
            None => return Err(UnregisteredComponent),
        };

        let handle = unsafe {
            self.alloc
                .alloc(obj)
                .expect("Failed to allocate object")
                .dissolve_data()
        };
        query.add_handle(handle);

        Ok(())
    }
}
