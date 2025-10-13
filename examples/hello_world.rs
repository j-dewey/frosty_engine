/*
 * A simplpe hello world to show how to add a system to
 * an app!
 */

use engine_core::app::WindowlessApp;
use engine_core::system::*;
use engine_core::{query::Query, SceneBuilder};
use frosty_alloc::FrostyAllocatable;

struct HelloWorldSystem {}
impl System for HelloWorldSystem {
    type Interop = Speaker;
    fn update(&self, mut objs: Query<Self::Interop>, _: u32) -> UpdateResult {
        for obj in objs.into_iter() {
            println!("{:?}", &obj.as_ref().text)
        }
        UpdateResult::CloseApp
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
