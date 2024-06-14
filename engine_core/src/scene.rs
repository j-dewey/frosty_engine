use frosty_alloc::Allocator;
use hashbrown::HashMap;

use crate::schedule::Schedule;

pub struct Scene {
    // this stores entities
    alloc: Allocator,
    // this stores systems
    schedule: Schedule,
}

impl Scene {
    pub fn new() -> Self {
        Scene {
            alloc: Allocator::new(),
            schedule: Schedule::new(),
        }
    }
}
