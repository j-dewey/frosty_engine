use basic_3d::render::general_3d_pipeline;
use basic_3d::{camera::Camera3d, GENERAL_3D_PACKAGE};
use cgmath::Rad;
use engine_core::assets::AssetManager;
use engine_core::{App, Entity, SceneBuilder};
use render::QUAD_INDEX_ORDER;
use render::{
    mesh::{IndexArray, Mesh},
    scheduled_pipeline::DEFAULT_MATERIAL_LABEL,
    vertex::MeshVertex,
    winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder},
};

fn generate_image_quad(camera: Camera3d) -> Entity {
    let verts = camera.generate_view_plane(1.0);
    let (neg_normal, _) = camera.forward_right();
    let tex_coords = [[0.0, 0.0], [0.0, 1.0], [1.0, 0.0], [1.0, 1.0]];

    Entity::new().chain_push_obj(Mesh {
        verts: verts
            .iter()
            .enumerate()
            .map(|(i, p)| MeshVertex {
                world_pos: (*p).into(),
                tex_coords: tex_coords[i],
                mat: 0,
                normal: (-neg_normal).into(),
            })
            .collect(),
        indices: IndexArray::new_u32(&QUAD_INDEX_ORDER),
        material: DEFAULT_MATERIAL_LABEL,
    })
}

fn set_scene(win_size: PhysicalSize<u32>) -> SceneBuilder {
    let camera = Camera3d::new_basic([0.0, 0.0, 0.0], Rad(0.0), Rad(0.0), win_size);

    let am = AssetManager::new("res/tests/asset_load/".into())
        .add("table_top.png")
        .expect("Failed to locate table_top.png");

    SceneBuilder::new()
        .register_package(GENERAL_3D_PACKAGE)
        .spawn(generate_image_quad(camera))
        .spawn_component(camera)
        .with_asset_manager(am)
        .prep_render_pipeline(&general_3d_pipeline)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let win_size = window.inner_size();
    App::new(&window).run_with_log(set_scene(win_size), event_loop, "logs/triangle.log");
}
