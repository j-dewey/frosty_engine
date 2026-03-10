use std::f32::consts::PI;

use basic_3d::render::general_3d_pipeline;
use basic_3d::{camera::Camera3d, GENERAL_3D_PACKAGE};
use cgmath::Rad;
use engine_core::{
    input,
    system::{System, UpdateResult},
    App, Entity, SceneBuilder,
};
use frosty_alloc::{FrostyAllocatable, NeedsSharedResource, SharedResource};
use render::{
    mesh::{IndexArray, Mesh},
    scheduled_pipeline::DEFAULT_MATERIAL_LABEL,
    vertex::MeshVertex,
    winit::{dpi::PhysicalSize, event_loop::EventLoop, window::WindowBuilder},
};

fn create_triangle_vertices(origin: [f32; 3], radius: f32, theta: f32) -> Vec<MeshVertex> {
    let mut verts = Vec::new();
    let factor = 2.0 * PI / 3.0;
    for i in 0..3 {
        let d_theta = i as i32 as f32;
        let dz = radius * (d_theta * factor + theta).cos();
        let dy = radius * (d_theta * factor + theta).sin();

        verts.push(MeshVertex {
            world_pos: [origin[0], origin[1] + dy, origin[2] + dz],
            tex_coords: [0.0, 0.0],
            mat: 0,
            normal: [1.0, 0.0, 0.0],
        });
    }

    verts
}

struct Rotater {
    speed: f32,
    theta: f32,
}
unsafe impl FrostyAllocatable for Rotater {}

struct Triangle {
    mesh: SharedResource<Mesh<MeshVertex>>,
    rotater: SharedResource<Rotater>,
    origin: [f32; 3],
    radius: f32,
}
impl Triangle {
    fn new(origin: [f32; 3], radius: f32) -> Self {
        unsafe {
            Self {
                mesh: SharedResource::new(),
                rotater: SharedResource::new(),
                origin,
                radius,
            }
        }
    }
}

unsafe impl FrostyAllocatable for Triangle {}
impl NeedsSharedResource for Triangle {
    fn shared_ids() -> Vec<frosty_alloc::AllocId>
    where
        Self: Sized,
    {
        vec![Mesh::<MeshVertex>::id(), Rotater::id()]
    }
    fn set_resources(&mut self, handles: Vec<frosty_alloc::ObjectHandleMut<u8>>) {
        self.mesh.set_handle(handles[0].cast_clone());
        self.rotater.set_handle(handles[1].cast_clone());
    }
}

struct TriangleRotater {}
impl System for TriangleRotater {
    type Interop = Triangle;

    fn update(
        &self,
        mut objs: engine_core::query::Query<Self::Interop>,
        thread: u32,
    ) -> UpdateResult {
        let dt = input::get_dt_seconds().expect("Failed to init input") as f32;

        while let Some(mut triangle) = objs.next(thread) {
            let mut_ref = triangle.as_mut();

            let mut rot_access = mut_ref
                .rotater
                .get_access_mut(thread)
                .expect("Failed to get access to Rotater");

            let rot = rot_access.as_mut();
            rot.theta += rot.speed * dt;
            let new_verts = create_triangle_vertices(mut_ref.origin, mut_ref.radius, rot.theta);

            mut_ref
                .mesh
                .get_access_mut(thread)
                .expect("Failed to get access to Mesh")
                .as_mut()
                .verts = new_verts;
        }
        UpdateResult::Skip
    }
}

fn spawn_triangle(origin: [f32; 3], radius: f32, speed: f32, initial_theta: f32) -> Entity {
    let verts = create_triangle_vertices(origin, radius, initial_theta);

    Entity::new()
        .chain_push_obj(Mesh {
            verts,
            indices: IndexArray::new_u32(&[0, 1, 2]),
            material: DEFAULT_MATERIAL_LABEL,
        })
        .chain_push_obj(Rotater {
            speed,
            theta: initial_theta,
        })
        .chain_push_handle_obj(Triangle::new(origin, radius))
}

fn set_scene(win_size: PhysicalSize<u32>) -> SceneBuilder {
    SceneBuilder::new()
        .register_package(GENERAL_3D_PACKAGE)
        .register_component::<Rotater>()
        .register_component::<Triangle>()
        .spawn_component(Camera3d::new_basic(
            [0.0, 0.0, 0.0],
            Rad(0.0),
            Rad(0.0),
            win_size,
        ))
        .spawn(spawn_triangle(
            [3.0, 0.5, 1.5],
            1.0,
            PI / 8.0,
            -30.0 * 2.0 * PI / 360.0,
        ))
        .spawn(spawn_triangle(
            [3.0, 0.5, -1.5],
            1.0,
            -PI / 4.0,
            -30.0 * 2.0 * PI / 360.0,
        ))
        .register_system(TriangleRotater {})
        .prep_render_pipeline(&general_3d_pipeline)
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let win_size = window.inner_size();
    App::new(&window).run_with_log(set_scene(win_size), event_loop, "logs/rotating_triangles");
}
