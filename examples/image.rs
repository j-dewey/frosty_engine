use std::f32::consts::FRAC_PI_8;

use basic_3d::camera::FlyCameraSystem;
use basic_3d::render::general_3d_pipeline;
use basic_3d::{camera::Camera3d, GENERAL_3D_PACKAGE};
use cgmath::Rad;
use engine_core::render_core::TextureLoader;
use engine_core::{App, Entity, SceneBuilder};
use render::scheduled_pipeline::ShaderLabel;
use render::QUAD_INDEX_ORDER;
use render::{
    mesh::{IndexArray, Mesh},
    vertex::MeshVertex,
    winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder},
};

fn generate_image_quad(camera: Camera3d) -> Entity {
    let label = ShaderLabel::from_string("image-texture".to_owned());
    let verts = camera.generate_view_plane(10.0);
    let (neg_normal, _) = camera.forward_right();
    let tex_coords = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];

    let diffuse_bytes = include_bytes!("res/image/table_top.png");
    let diffuse_image = image::load_from_memory(diffuse_bytes).unwrap();
    let rgba = diffuse_image.to_rgba8();

    use image::GenericImageView;
    let dimensions = diffuse_image.dimensions();

    Entity::new()
        .chain_push_obj(Mesh {
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
            material: label.clone(),
        })
        .chain_push_obj(TextureLoader {
            label,
            width: dimensions.0,
            height: dimensions.1,
            pixel_data: rgba.into_raw(),
        })
}

fn set_scene(win_size: PhysicalSize<u32>) -> SceneBuilder {
    let camera = Camera3d::new_basic([0.0, 0.0, 0.0], Rad(0.0), Rad(0.0), win_size);
    unsafe {
        FlyCameraSystem::load_controls().expect("Failed to load FlyCamera controls.");
    }

    SceneBuilder::new()
        .register_package(GENERAL_3D_PACKAGE)
        .spawn(generate_image_quad(camera))
        .spawn_component(camera)
        .register_system(FlyCameraSystem {
            speed: 10.0,
            rot_speed: Rad(FRAC_PI_8 * 2.0),
        })
        .prep_render_pipeline(&general_3d_pipeline)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let win_size = window.inner_size();
    App::new(&window).run_with_log(set_scene(win_size), event_loop, "logs/triangle.log");
}
