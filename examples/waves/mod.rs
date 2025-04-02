use engine_core::{render_core::DynamicRenderPipeline, App, SceneBuilder, Spawner};
use render::{
    window_state::WindowState,
    winit::{event_loop::EventLoop, window::WindowBuilder},
};

mod comps;
use comps::register_comps;

fn set_up_rendering(alloc: &mut Spawner, ws: &WindowState) -> DynamicRenderPipeline {
    DynamicRenderPipeline::new_empty()
}

fn set_up_scene() -> SceneBuilder {
    SceneBuilder::new()
        .register_components(register_comps)
        .prep_render_pipeline(&set_up_rendering)
}

pub(crate) fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    App::new(&window).run(set_up_scene(), event_loop);
}
