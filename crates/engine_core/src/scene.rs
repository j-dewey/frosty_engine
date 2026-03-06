use frosty_alloc::{debug::DebugOutter, FrostyAllocatable};
use render::window_state::WindowState;

use crate::{
    assets::{pending::MeshFile, AssetManager},
    package::Package,
    render_core::DynamicRenderPipeline,
    schedule::Schedule,
    system::SystemInterface,
    Entity, Spawner, MASTER_THREAD,
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

type PipelineInitFn = &'static dyn Fn(&mut Spawner, &WindowState) -> DynamicRenderPipeline;

pub struct SceneBuilder {
    // this stores entities
    pub(crate) alloc: Spawner,
    // this stores assets
    assets: AssetManager,
    // this stores systems
    schedule: Schedule,
    // this stores rendering
    rendering: Option<PipelineInitFn>,
}

impl SceneBuilder {
    pub fn new() -> Self {
        Self {
            alloc: Spawner::new(),
            assets: AssetManager::new("".to_owned()),
            schedule: Schedule::new(),
            rendering: None,
        }
    }

    pub fn with_asset_manager(mut self, am: AssetManager) -> Self {
        self.assets = am;
        self
    }

    pub fn get_mut_spawner(&mut self) -> &mut Spawner {
        &mut self.alloc
    }

    pub fn register_components<F: FnOnce(Self) -> Self>(self, registration_fn: F) -> Self {
        (registration_fn)(self)
    }

    pub fn register_component<C: 'static + FrostyAllocatable>(mut self) -> Self {
        self.alloc.register_component::<C>();
        self
    }

    pub fn register_system<S: SystemInterface>(mut self, system: S) -> Self {
        self.schedule.add_system(system, &mut self.alloc);
        self
    }

    pub fn register_package<const N: usize>(mut self, package: Package<N>) -> Self {
        package.register_all(&mut self.alloc);
        self
    }

    pub fn spawn(mut self, entity: Entity) -> Self {
        self.alloc
            .spawn(entity)
            .expect("Not all components were registered!");
        self
    }

    pub fn spawn_component<C: 'static + FrostyAllocatable>(mut self, comp: C) -> Self {
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

    pub fn build(mut self, ws: &WindowState) -> Scene {
        // alert the asset manager of all pending asset objects

        if let Some(meshes) = self.alloc.get_query::<MeshFile>(MASTER_THREAD) {
            self.assets.read_query(meshes, MASTER_THREAD);
        }

        let resource_stream = self
            .assets
            .read(&ws.bindings)
            .expect("Failed to load an asset");

        let rendering = (self.rendering.unwrap())(&mut self.alloc, ws);
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

impl DebugOutter for SceneBuilder {
    fn dump_data(&self, fs: &mut std::fs::File) {
        self.alloc.dump_data(fs);
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

impl DebugOutter for Scene {
    fn dump_data(&self, fs: &mut std::fs::File) {
        self.alloc.dump_data(fs);
    }
}
