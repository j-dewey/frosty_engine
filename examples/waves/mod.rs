use engine_core::{render_core::DynamicRenderPipeline, App, Scene, Spawner};
use render::winit::{event_loop::EventLoop, window::WindowBuilder};

fn component_registration(app: App<'_>) -> App<'_> {
    app
}

fn set_up_rendering(alloc: &mut Spawner) -> DynamicRenderPipeline {
    todo!()
}

fn set_up_scene() -> Scene {
    Scene::new(set_up_rendering)
}

pub(crate) fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    App::new(set_up_scene(), &window)
        .register_components(component_registration)
        .register_rendering()
        .run(event_loop);
}
