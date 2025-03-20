use frosty_alloc::FrostyAllocatable;

use crate::{
    render_core::DynamicRenderPipeline, schedule::Schedule, system::SystemInterface, Spawner,
};

// A Scene defines which entities are available, which systems are active, and how rendering should occur.
// This clearly has a major issue:
//      The render pipeline depends on entities already being added to the scene
// A simple solution would be to have the Scene constructor take in a spawner and renderer:
//      fn set_up_scene() -> Scene{
//          let spawner = Spawner::new()
//              .register_component(..)
//              .register_component(..)
//              .spawn(..)
//              .spawn(..)
//              .spawn(..);
//          let pipeline = set_up_pipeline(&spawner);
//          Scene::new(spawner, pipeline)
//      }
// This is clunky though and the following is prefered:
//      fn set_up_scene() -> Scene{
//          SceneBuilder::new()
//              .register_component(..)
//              .register_component(..)
//              .spawn(..)
//              .spawn(..)
//              .spawn(..)
//              .add_pipeline( pipeline_init_fn )
//              .build()
//      }
// Unfortunately, initializing the pipeline will at some point require access to a WindowState
// which will not exist until App is initialized, so the best to achieve is
//      fn set_up_scene() -> SceneBuilder {
//          SceneBuilder::new()
//              .register_component(..)
//              .register_component(..)
//              .spawn(..)
//              .spawn(..)
//              .spawn(..)
//              .add_pipeline( pipeline_init_fn )
//      }

type PipelineInitFn = &'static dyn Fn(&mut Spawner) -> DynamicRenderPipeline;

pub struct SceneBuilder {
    // this stores entities
    alloc: Spawner,
    // this stores systems
    schedule: Schedule,
    // this stores rendering
    rendering: Option<PipelineInitFn>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            alloc: Spawner::new(),
            schedule: Schedule::new(),
            rendering: None,
        }
    }

    pub fn get_mut_spawner(&mut self) -> &mut Spawner {
        &mut self.alloc
    }

    pub fn register_components<F: FnOnce(Self) -> Self>(self, registration_fn: F) -> Self {
        (registration_fn)(self)
    }

    pub fn register_component<C: FrostyAllocatable>(mut self) -> Self {
        self.alloc.register_component::<C>();
        self
    }

    pub fn register_system<S: SystemInterface>(mut self, system: S) -> Self {
        self.schedule.add_system(system, &mut self.alloc);
        self
    }

    pub fn spawn_component<C: FrostyAllocatable>(mut self, comp: C) -> Self {
        if !self.alloc.is_registered::<C>() {
            self.alloc.register_component::<C>();
        }
        self.alloc
            .spawn_obj(comp)
            .expect("Component appears registered but is not");
        self
    }

    pub fn prep_render_pipeline(mut self, render_init_fn: PipelineInitFn) -> Self {
        self.rendering = Some(render_init_fn);
        self
    }

    pub fn build(mut self) -> Scene {
        let rendering = (self.rendering.unwrap())(&mut self.alloc);
        Scene {
            alloc: self.alloc,
            schedule: self.schedule,
            rendering,
        }
    }

    pub(crate) fn dissolve(self) -> (Spawner, Schedule, Option<PipelineInitFn>) {
        (self.alloc, self.schedule, self.rendering)
    }
}

pub struct Scene {
    // this stores entities
    alloc: Spawner,
    // this stores systems
    schedule: Schedule,
    // this stores rendering
    rendering: DynamicRenderPipeline,
}

impl Scene {
    pub(crate) fn get_mutable_parts(
        &mut self,
    ) -> (&mut Spawner, &mut Schedule, &mut DynamicRenderPipeline) {
        (&mut self.alloc, &mut self.schedule, &mut self.rendering)
    }

    pub fn get_mut_spawner(&mut self) -> &mut Spawner {
        &mut self.alloc
    }
}
