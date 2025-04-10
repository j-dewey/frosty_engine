/*
 * A simplpe hello world to show how to add a system to
 * an app!
 */

use std::any::TypeId;

use engine_core::app::WindowlessApp;
use engine_core::system::*;
use engine_core::{query::Query, SceneBuilder};
use frosty_alloc::{AllocId, FrostyAllocatable};

struct HelloWorldSystem {}
impl System for HelloWorldSystem {
    type Interop = Speaker;
    fn update(&self, mut objs: Query<Self::Interop>) -> UpdateResult {
        for obj in objs.into_iter() {
            println!("{:?}", &obj.as_ref().text)
        }
        UpdateResult::CloseApp
    }
}
impl SystemInterface for HelloWorldSystem {
    fn id() -> SystemId
    where
        Self: Sized,
    {
        SystemId(0)
    }

    fn start_update(&self, objs: Query<u8>) -> UpdateResult {
        let real_objs = unsafe { objs.cast::<Speaker>() };
        self.update(real_objs)
    }

    fn alloc_id(&self) -> TypeId {
        Speaker::id()
    }

    fn query_type() -> SystemQuerySchedule
    where
        Self: Sized,
    {
        SystemQuerySchedule::Update
    }

    fn dependencies() -> Vec<SystemId>
    where
        Self: Sized,
    {
        vec![]
    }
}

struct Speaker {
    text: String,
}
unsafe impl FrostyAllocatable for Speaker {}

fn main() {
    let scene = SceneBuilder::new()
        .register_component::<Speaker>()
        .spawn_component(Speaker {
            text: "Hello World!".into(),
        })
        .spawn_component(Speaker {
            text: "Bark!".into(),
        })
        .register_system(HelloWorldSystem {});

    WindowlessApp::new().run(scene);
}
