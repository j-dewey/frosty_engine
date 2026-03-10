use frosty_alloc::{AllocId, FrostyAllocatable};

use crate::{system::SystemInterface, Spawner};

// A function pointer that registers
//type RegistrationFunction = &'static dyn FnOnce(&mut Spawner);
type RegistrationFunction = fn(&mut Spawner);
pub const fn register<C: FrostyAllocatable>() -> (AllocId, RegistrationFunction) {
    (AllocId::of::<C>(), {
        |alloc: &mut Spawner| alloc.register_component::<C>()
    })
}

// There are a lot of [Component]s which are so dependent on certain [System]s that
// they serve no function without them. There are also a lot of [System]s which are
// fundamental to making a game (i.e. having some camera controller).
//
// Packages allow all of these to be set up under one easy umbrella
// Packages follow the builder pattern
pub struct Package<const N: usize> {
    components: [(AllocId, RegistrationFunction); N],
}

impl<const N: usize> Package<N> {
    pub const fn new(components: [(AllocId, RegistrationFunction); N]) -> Self {
        Self { components }
    }

    pub fn register_all(self, alloc: &mut Spawner) {
        for (_, reg_fn) in &self.components {
            reg_fn(alloc);
        }
    }

    pub fn disable_component<C: FrostyAllocatable>(self) {}

    pub fn disable_system<S: SystemInterface>(self) {}
}
