use wgpu::{core::device, util::DeviceExt};

use crate::window_state::WindowState;

use super::shader::ShaderGroup;

pub struct ScaleFactor {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl From<f32> for ScaleFactor {
    fn from(value: f32) -> Self {
        Self {
            x: value,
            y: value,
            z: value,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct MeshVertex {
    pub world_pos: [f32; 3],  // x, y, z
    pub tex_coords: [f32; 2], // more information will be needed when lighting is implemented
    pub mat: u32,
    pub normal: [f32; 3],
}

impl MeshVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<MeshVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // world pos
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // for @location(n) when defining struct in shader
                    format: wgpu::VertexFormat::Float32x3,
                },
                // color
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1, // location(1)
                    format: wgpu::VertexFormat::Float32x2,
                },
                // material
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>() + std::mem::size_of::<[f32; 2]>())
                        as wgpu::BufferAddress,
                    shader_location: 2, // location(2)
                    format: wgpu::VertexFormat::Uint32,
                },
                // normal
                wgpu::VertexAttribute {
                    offset: (std::mem::size_of::<[f32; 3]>()
                        + std::mem::size_of::<[f32; 2]>()
                        + std::mem::size_of::<u32>())
                        as wgpu::BufferAddress,
                    shader_location: 3, // location(3)
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

pub fn max_dist(verts: &Vec<MeshVertex>) -> cgmath::Vector3<f32> {
    let min_x = f32::MAX;
    let min_y = f32::MAX;
    let min_z = f32::MAX;
    let max_x = f32::MIN;
    let max_y = f32::MIN;
    let max_z = f32::MAX;
    for v in verts {
        // no switch statemente
    }
    todo!()
}

pub fn transform_verts(verts: &mut Vec<MeshVertex>, delta: [f32; 3]) {
    for v in verts {
        v.world_pos = [
            v.world_pos[0] + delta[0],
            v.world_pos[1] + delta[1],
            v.world_pos[2] + delta[2],
        ]
    }
}

pub fn scale_verts_from_point(
    verts: &mut Vec<MeshVertex>,
    factor: impl Into<ScaleFactor>,
    point: [f32; 3],
) {
    let scale: ScaleFactor = factor.into();
    for v in verts {
        let base_vector = cgmath::Vector3::new(
            v.world_pos[0] - point[0],
            v.world_pos[1] - point[1],
            v.world_pos[2] - point[2],
        );
        v.world_pos = [
            point[0] + base_vector.x * scale.x,
            point[1] + base_vector.y * scale.y,
            point[2] + base_vector.z * scale.z,
        ];
    }
}

pub fn rotate_verts_about_line(
    _verts: &Vec<MeshVertex>,
    _line: cgmath::Vector3<f32>,
    _rotation: cgmath::Rad<f32>,
) {
    todo!()
}

pub struct ProtoMesh {
    pub verts: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub mat: u32,
}

pub struct RawMesh {
    pub vertices: Vec<MeshVertex>,
    pub indices: Vec<u32>,
    pub v_buf: wgpu::Buffer,
    pub i_buf: wgpu::Buffer,
    pub mat: u32,
}

impl RawMesh {
    pub fn new(
        vertices: Vec<MeshVertex>,
        indices: Vec<u32>,
        mat: u32,
        device: &wgpu::Device,
    ) -> Self {
        let i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices[..]),
            usage: wgpu::BufferUsages::INDEX,
        });

        let v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GUIMesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            vertices,
            indices,
            v_buf,
            i_buf,
            mat,
        }
    }

    pub fn translate(&mut self, delta: [f32; 3]) {
        transform_verts(&mut self.vertices, delta)
    }

    pub fn reload_buffers(&mut self, device: &wgpu::Device) {
        self.i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&self.indices[..]),
            usage: wgpu::BufferUsages::INDEX,
        });

        self.v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GUIMesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&self.vertices[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });
    }

    pub fn get_verts(&self) -> &[MeshVertex] {
        &self.vertices[..]
    }

    pub fn get_indices(&self) -> &[u32] {
        &self.indices[..]
    }

    pub fn get_shader_group(&self) -> ShaderGroup {
        ShaderGroup::new_borrowed(
            &self.v_buf,
            &self.i_buf,
            self.mat,
            self.indices.len() as u32,
        )
    }
}

pub enum Mesh {
    Static { mesh: RawMesh },
    Dynamic { mesh: RawMesh },
    StateUpdated { mesh: RawMesh, reload: bool },
}

impl Mesh {
    pub fn new_static(mesh: RawMesh) -> Self {
        Self::Static { mesh }
    }

    pub fn new_dynamic(mesh: RawMesh) -> Self {
        Self::Dynamic { mesh }
    }

    pub fn new_state_updated(mesh: RawMesh) -> Self {
        Self::StateUpdated {
            mesh,
            reload: false,
        }
    }

    pub fn get_raw(&self) -> &RawMesh {
        match self {
            Self::Static { ref mesh }
            | Self::Dynamic { ref mesh }
            | Self::StateUpdated { ref mesh, .. } => mesh,
        }
    }

    pub fn get_raw_mut(&mut self) -> &mut RawMesh {
        match self {
            Self::Static { ref mut mesh }
            | Self::Dynamic { ref mut mesh }
            | Self::StateUpdated { ref mut mesh, .. } => mesh,
        }
    }

    pub fn get_shader_group(&mut self, device: &wgpu::Device) -> ShaderGroup {
        match self {
            Self::Static { ref mesh }
            | Self::StateUpdated {
                ref mesh,
                reload: false,
            } => mesh.get_shader_group(),
            Self::Dynamic { ref mut mesh }
            | Self::StateUpdated {
                ref mut mesh,
                reload: true,
            } => {
                mesh.reload_buffers(device);
                mesh.get_shader_group()
            }
        }
    }
}
