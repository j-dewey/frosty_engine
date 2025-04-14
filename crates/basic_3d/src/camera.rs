use cgmath::*;
use engine_core::render_core::GivesBindGroup;
use frosty_alloc::FrostyAllocatable;
use render::wgpu::util::DeviceExt;
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

    pub fn new_basic<V: Into<Point3<f32>>>(position: V, window_size: PhysicalSize<u32>) -> Self {
        Self {
            position: position.into(),
            yaw: cgmath::Rad(0.0),
            pitch: cgmath::Rad(0.0),
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

    pub fn move_rotate(&mut self, translate: [f32; 2], rotate: [f32; 2], dt: f32) {
        let (forward, right) = self.forward_right();
        let d_forward = forward * translate[0] * dt;
        let d_right = right * translate[1] * dt;
        let d_yaw = rotate[0] * dt;
        let d_pitch = rotate[1] * dt;

        self.position += d_forward + d_right;
        self.yaw += cgmath::Rad(d_yaw);
        self.pitch += cgmath::Rad(d_pitch);
    }
}

unsafe impl bytemuck::Pod for Camera3d {}
unsafe impl bytemuck::Zeroable for Camera3d {}

unsafe impl FrostyAllocatable for Camera3d {}

impl GivesBindGroup for Camera3d {
    fn get_bind_group_layout(&self, ws: &WindowState) -> wgpu::BindGroupLayout {
        ws.device
            .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Snow Details"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            })
    }
    fn get_bind_group(&self, ws: &WindowState) -> wgpu::BindGroup {
        let view_matrix: [[f32; 4]; 4] =
            (self.projection.calc_matrix() * self.calc_matrix()).into();

        let camera_bind_group_layout =
            ws.device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    entries: &[wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    }],
                    label: Some("camera_bind_group_layout"),
                });

        let camera_buffer = ws
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[view_matrix]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });
        ws.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        })
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
