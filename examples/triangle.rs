use basic_3d::{camera::Camera3d, mesh::Mesh3d};
use engine_core::{
    render_core::{layout::ShaderNodeLayout, DynamicRenderPipeline},
    App, SceneBuilder, Spawner,
};
use render::{
    mesh::Mesh,
    window_state::WindowState,
    winit::{event_loop::EventLoop, window::WindowBuilder},
};

fn prep_render(alloc: &mut Spawner, ws: &WindowState) -> DynamicRenderPipeline {
    DynamicRenderPipeline::new_empty().register_shader::<Mesh>(todo!(), ws, alloc)
}

fn set_scene() -> SceneBuilder {
    SceneBuilder::new()
        .register_component::<Camera3d>()
        .prep_render_pipeline(&prep_render)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    App::new(&window).run(set_scene(), event_loop);
}
