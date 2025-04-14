use std::any::TypeId;

use frosty_alloc::{Allocator, FrostyAllocatable, ObjectHandleMut};
use hashbrown::HashMap;

use crate::{
    query::{Query, QueryForm, RawQuery},
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
    queries: HashMap<TypeId, RawQuery>,
    registered_components: HashMap<TypeId, ConverterFn>,
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

    pub fn is_registered<C: FrostyAllocatable>(&mut self) -> bool {
        self.registered_components.get(&C::id()).is_some()
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

    pub(crate) fn get_raw_query(&mut self, id: &TypeId) -> Option<&mut RawQuery> {
        self.queries.get_mut(id)
    }

    pub fn get_query<C: FrostyAllocatable>(&self, thread: u32) -> Option<Query<C>> {
        let raw = self.queries.get(&C::id())?;
        Some(Query::new(raw, thread))
    }

    pub fn get_query_by_id(&self, id: &TypeId, thread: u32) -> Option<Query<u8>> {
        let raw = self.queries.get(id)?;
        Some(Query::new(raw, thread))
    }

    pub fn get_dissolved_query(&self, id: TypeId, thread: u32) -> Option<Query<u8>> {
        let raw = self.queries.get(&id)?;
        Some(Query::new(raw, thread))
    }
}

#[cfg(test)]
mod spawner_test {
    use frosty_alloc::FrostyAllocatable;

    use crate::{query::Query, Spawner};

    #[test]
    fn spawn_generic() {
        #[derive(Debug, Clone, Copy)]
        struct Generic<T>(T);
        unsafe impl<T: 'static> FrostyAllocatable for Generic<T> {}

        let mut spawner = Spawner::new();
        spawner.register_component::<Generic<i32>>();
        spawner.register_component::<Generic<f32>>();
        spawner
            .spawn_obj(Generic(6i32))
            .expect("Failed to register Generic<i32>");
        spawner
            .spawn_obj(Generic(12.0f32))
            .expect("Failed to register Generic<f32>");

        let mut ints: Query<Generic<i32>> = unsafe {
            spawner
                .get_query_by_id(&Generic::<i32>::id(), 0)
                .expect("Failed to load Generic<i32> Query")
                .cast()
        };
        let mut floats: Query<Generic<f32>> = unsafe {
            spawner
                .get_query_by_id(&Generic::<f32>::id(), 0)
                .expect("Failed to load Generic<f32> Query")
                .cast()
        };

        assert_eq!(
            ints.next(0)
                .expect("Failed to load initial element for Generic<i32> Query")
                .as_ref()
                .0,
            6
        );
        assert!(ints.next(0).is_none(), "Too many ints read from Query");

        assert_eq!(
            floats
                .next(0)
                .expect("Failed to load initial element for Generic<i32> Query")
                .as_ref()
                .0,
            12.0
        );
        assert!(floats.next(0).is_none(), "Too many ints read from Query");
    }
}
