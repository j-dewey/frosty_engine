use basic_3d::camera::Camera3d;
use basic_3d::render::general_3d_pipeline;
use engine_core::{App, SceneBuilder};
use render::{
    mesh::{IndexArray, Mesh},
    vertex::MeshVertex,
    winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder},
};

fn generate_triangle() -> Mesh<MeshVertex> {
    let top = MeshVertex {
        world_pos: [0.0, 1.0, 0.5],
        tex_coords: [0.0, 0.0],
        mat: 0,
        normal: [0.0, 0.0, 0.0],
    };
    let left = MeshVertex {
        world_pos: [-0.5, 1.0, 0.5],
        tex_coords: [0.0, 0.0],
        mat: 0,
        normal: [0.0, 0.0, 0.0],
    };
    let right = MeshVertex {
        world_pos: [0.5, 1.0, 0.5],
        tex_coords: [0.0, 0.0],
        mat: 0,
        normal: [0.0, 0.0, 0.0],
    };
    Mesh {
        verts: vec![top, right, left],
        indices: IndexArray::new_u32(&[0, 2, 1]),
    }
}

fn set_scene(win_size: PhysicalSize<u32>) -> SceneBuilder {
    SceneBuilder::new()
        .register_component::<Camera3d>()
        .register_component::<Mesh<MeshVertex>>()
        .spawn_component(Camera3d::new_basic([0.0, 0.0, 0.0], win_size))
        .spawn_component(generate_triangle())
        .prep_render_pipeline(&general_3d_pipeline)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let win_size = window.inner_size();
    App::new(&window).run(set_scene(win_size), event_loop);
}
