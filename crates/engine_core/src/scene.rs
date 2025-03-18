use frosty_alloc::Allocator;

use crate::{schedule::Schedule, Spawner};

pub struct Scene {
    // this stores entities
    alloc: Spawner,
    // this stores systems
    schedule: Schedule,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            alloc: Spawner::new(),
            schedule: Schedule::new(),
        }
    }

    pub fn get_mutable_parts(&mut self) -> (&mut Spawner, &mut Schedule) {
        (&mut self.alloc, &mut self.schedule)
    }
}
