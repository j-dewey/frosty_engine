use engine_core::{App, Scene};
use render::winit::{event_loop::EventLoop, window::WindowBuilder};

fn component_registration(app: App<'_>) -> App<'_> {
    app
}

fn set_up() -> Scene {
    Scene::new()
}

pub(crate) fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    App::new(set_up(), &window)
        .register_components(component_registration)
        .run(event_loop);
}
