use engine_core::{App, Scene};
use render::winit::{event_loop::EventLoop, window::WindowBuilder};

fn set_up() -> Scene {
    Scene::new()
}

pub(crate) fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    App::new(set_up(), &window).run(event_loop);
}
