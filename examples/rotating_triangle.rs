use std::f32::consts::PI;

use basic_3d::camera::Camera3d;
use basic_3d::render::general_3d_pipeline;
use cgmath::{Basis2, InnerSpace, Rad, Rotation, Rotation2, Vector2};
use engine_core::{
    input,
    system::{System, SystemId, SystemInterface, UpdateResult},
    App, SceneBuilder, MASTER_THREAD,
};
use frosty_alloc::FrostyAllocatable;
use render::{
    mesh::{IndexArray, Mesh},
    vertex::MeshVertex,
    winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder},
};

struct TriangleRotater {
    speed: f32,
}

impl System for TriangleRotater {
    type Interop = Mesh<MeshVertex>;

    fn update(&self, mut objs: engine_core::query::Query<Self::Interop>) -> UpdateResult {
        let dt = input::get_dt_seconds().expect("Failed to init input") as f32;
        while let Some(mut triangle) = objs.next(MASTER_THREAD) {
            let mut_ref = triangle.as_mut();

            let as_vec = Vector2::new(mut_ref.verts[0].world_pos[2], mut_ref.verts[0].world_pos[1]);
            let cos_theta = as_vec.dot(Vector2::unit_x());
            let mut old_theta = cos_theta.acos();
            // acos is only defined for positive y
            if mut_ref.verts[0].world_pos[1] <= 0.0 {
                old_theta = 2.0 * PI - old_theta;
            }
            let theta = dt * self.speed + old_theta;

            let x_offset = mut_ref.verts[0].world_pos[0];

            println!("\ndt:        {:?}", dt);
            println!("position : {:?}", mut_ref.verts[0].world_pos);
            println!("cos_theta: {:?}", cos_theta);
            println!("d_theta:   {:?}", dt * self.speed);
            println!("old_theta: {:?}", old_theta);
            println!("new_theta: {:?}", theta);

            // update points
            for i in 0..3 {
                let rot: Basis2<f32> =
                    Rotation2::from_angle(Rad(theta + PI * (2.0 / 3.0) * i as i32 as f32));
                let new_pos = rot.rotate_vector(Vector2::unit_x());
                mut_ref.verts[i].world_pos = [x_offset, new_pos.y, new_pos.x];
            }
        }
        UpdateResult::Skip
    }
}

impl SystemInterface for TriangleRotater {
    fn start_update(&self, objs: engine_core::query::Query<u8>) -> UpdateResult {
        self.update(unsafe { objs.cast() })
    }
    fn dependencies() -> Vec<SystemId>
    where
        Self: Sized,
    {
        vec![]
    }
    fn id() -> SystemId
    where
        Self: Sized,
    {
        SystemId(0)
    }
    fn alloc_id(&self) -> std::any::TypeId {
        Mesh::<MeshVertex>::id()
    }
}

fn generate_triangle() -> Mesh<MeshVertex> {
    let top = MeshVertex {
        world_pos: [2.5, 0.0, 1.0],
        tex_coords: [0.0, 0.0],
        mat: 0,
        normal: [0.0, 0.0, 0.0],
    };
    let left = MeshVertex {
        world_pos: [2.5, 1.0, -1.0],
        tex_coords: [0.0, 0.0],
        mat: 0,
        normal: [0.0, 0.0, 0.0],
    };
    let right = MeshVertex {
        world_pos: [2.5, -1.0, -1.0],
        tex_coords: [0.0, 0.0],
        mat: 0,
        normal: [0.0, 0.0, 0.0],
    };
    Mesh {
        verts: vec![top, left, right],
        indices: IndexArray::new_u32(&[0, 1, 2]),
    }
}

fn set_scene(win_size: PhysicalSize<u32>) -> SceneBuilder {
    SceneBuilder::new()
        .register_component::<Camera3d>()
        .register_component::<Mesh<MeshVertex>>()
        .spawn_component(Camera3d::new_basic([0.0, 0.0, 0.0], win_size))
        .spawn_component(generate_triangle())
        .register_system(TriangleRotater {
            speed: PI as f32 / 4.0,
        })
        .prep_render_pipeline(&general_3d_pipeline)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let win_size = window.inner_size();
    App::new(&window).run(set_scene(win_size), event_loop);
}
