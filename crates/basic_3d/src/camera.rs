use cgmath::*;
use engine_core::input::{self, BackwardAction, ForwardAction, LeftAction, RightAction};
use engine_core::query::Query;
use engine_core::render_core::GivesBindGroup;
use engine_core::system::{System, UpdateResult};
use frosty_alloc::FrostyAllocatable;
use render::winit::dpi::PhysicalSize;
use render::{wgpu, window_state::WindowState};

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

//const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug, Copy, Clone)]
pub struct Camera3d {
    position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    projection: Projection,
}

impl Camera3d {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
        projection: Projection,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection,
        }
    }

    pub fn new_basic<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
        window_size: PhysicalSize<u32>,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
            projection: Projection::new(
                window_size.width,
                window_size.height,
                cgmath::Deg(45.0),
                0.1,
                100.0,
            ),
        }
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        let (sin_pitch, cos_pitch) = self.pitch.0.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.0.sin_cos();

        Matrix4::look_to_rh(
            self.position,
            Vector3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw).normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn forward_right(&self) -> (Vector3<f32>, Vector3<f32>) {
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        (forward, right)
    }

    // move and rotate self by some deltas
    pub fn move_rotate(&mut self, translate: Vector3<f32>, rotate: [f32; 2], dt: f32) {
        let d_yaw = rotate[0] * dt;
        let d_pitch = rotate[1] * dt;

        self.position += translate;
        self.yaw += cgmath::Rad(d_yaw);
        self.pitch += cgmath::Rad(d_pitch);
    }

    // set position
    pub fn move_to(&mut self, position: Point3<f32>) {
        self.position = position;
    }

    // set rotations
    pub fn rotate_to(&mut self, rotation: Euler<Rad<f32>>) {
        self.yaw = rotation.x;
        self.pitch = rotation.y;
    }
}

unsafe impl bytemuck::Pod for Camera3d {}
unsafe impl bytemuck::Zeroable for Camera3d {}

unsafe impl FrostyAllocatable for Camera3d {}

impl GivesBindGroup for Camera3d {
    fn get_bind_group_layout(ws: &WindowState) -> wgpu::BindGroupLayout {
        ws.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Camera3D"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }

    fn get_uniform_data(&self) -> Box<[u8]> {
        let view_matrix: [[f32; 4]; 4] =
            (self.projection.calc_matrix() * self.calc_matrix()).into();
        let matrix_bytes: &[u8] = bytemuck::cast_slice(&view_matrix[..]);
        println!("{:?}", view_matrix);
        Box::from(&matrix_bytes[..])
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Projection {
    aspect: f32,
    fovy: Rad<f32>,
    pub znear: f32,
    pub zfar: f32,
}

impl Projection {
    pub fn new<F: Into<Rad<f32>>>(width: u32, height: u32, fovy: F, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy: fovy.into(),
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        OPENGL_TO_WGPU_MATRIX * perspective(self.fovy, self.aspect, self.znear, self.zfar)
    }
}

pub struct FlyCameraSystem {
    pub speed: f32,
    pub rot_speed: Rad<f32>,
}

impl System for FlyCameraSystem {
    type Interop = Camera3d;

    // This should only applly to the first Camera in the scene
    fn update(&self, mut objs: Query<Self::Interop>, thread: u32) -> UpdateResult {
        let mut ptr = match objs.next(thread) {
            Some(cam) => cam,
            None => return UpdateResult::Skip,
        };
        let cam = ptr.as_mut();

        let dt = input::get_dt_seconds().expect("Failed to init input handler") as f32;

        let d_forward = input::get_axis::<ForwardAction, BackwardAction>() as f32 * self.speed * dt;
        let d_right = input::get_axis::<RightAction, LeftAction>() as f32 * self.speed * dt;
        let (forward, right) = cam.forward_right();
        let delta = d_forward * forward + d_right * right;

        cam.position += delta;

        UpdateResult::Skip
    }
}
