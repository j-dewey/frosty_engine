use frosty_alloc::Allocator;

use crate::{render_core::DynamicRenderPipeline, schedule::Schedule, Spawner};

pub struct Scene {
    // this stores entities
    alloc: Spawner,
    // this stores systems
    schedule: Schedule,
    // this stores rendering
    rendering: DynamicRenderPipeline,
}

impl Scene {
    pub fn new<F: FnOnce(&mut Spawner) -> DynamicRenderPipeline>(render_set_up: F) -> Self {
        let mut alloc = Spawner::new();
        let rendering = render_set_up(&mut alloc);
        Scene {
            alloc,
            schedule: Schedule::new(),
            rendering,
        }
    }

    pub fn get_mutable_parts(&mut self) -> (&mut Spawner, &mut Schedule) {
        (&mut self.alloc, &mut self.schedule)
    }

    pub fn get_mut_spawner(&mut self) -> &mut Spawner {
        &mut self.alloc
    }
}
