use anonymous_table::Anonymous;
use cgmath::*;
use std::alloc;
use std::f32::consts::FRAC_PI_2;
use std::ptr;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: cgmath::Matrix4<f32> = cgmath::Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

//const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct Camera {
    pub position: Point3<f32>,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
}

impl Camera {
    pub fn new<V: Into<Point3<f32>>, Y: Into<Rad<f32>>, P: Into<Rad<f32>>>(
        position: V,
        yaw: Y,
        pitch: P,
    ) -> Self {
        Self {
            position: position.into(),
            yaw: yaw.into(),
            pitch: pitch.into(),
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

impl Anonymous for Camera {
    fn id() -> u16 {
        // bascially just gotta be under 20
        15
    }
}

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

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct PodCameraPtr(*mut Camera);
unsafe impl bytemuck::Zeroable for PodCameraPtr {}
unsafe impl bytemuck::Pod for PodCameraPtr {}

impl PodCameraPtr {
    pub fn new(raw: Camera) -> Self {
        let layout = alloc::Layout::new::<Camera>();
        unsafe {
            let raw_ptr = alloc::alloc(layout) as *mut Camera;
            ptr::write(raw_ptr, raw);
            Self(raw_ptr)
        }
    }

    pub fn get_ref(&self) -> &Camera {
        unsafe { self.0.as_ref().unwrap() }
    }

    pub fn move_ip(&self, change: cgmath::Vector3<f32>) {
        let cam_mut = unsafe { self.0.as_mut().unwrap() };
        cam_mut.position = cgmath::Point3 {
            x: cam_mut.position.x + change.x,
            y: cam_mut.position.y + change.y,
            z: cam_mut.position.z + change.z,
        };
    }

    pub fn rotate_ip(&self, change: cgmath::Vector2<f32>) {
        let cam_mut = unsafe { self.0.as_mut().unwrap() };
        cam_mut.yaw += Rad(change.x);
        cam_mut.pitch += Rad(change.y);
    }
}
