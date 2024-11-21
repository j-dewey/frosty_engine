/*
 * A simplpe hello world to show how to add a system to
 * an app!
 */

use engine_core::{App, System, SystemId, SystemInterface};

struct HelloWorldSystem {}

impl System for HelloWorldSystem {
    type Interop = String;
    fn update(&mut self, objs: Query<Self::Interop>) {
        print!(objs)
    }
}

fn main() {
    let mut scene = Scene::new()
        .add_system(HelloWorldSystem {})
        .spawn_entity(Entity.single_obj("Hello World!"));

    App::new().start_with(scene).run();
}
