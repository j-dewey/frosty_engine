use wgpu::util::DeviceExt;

use super::shader::ShaderGroup;
use super::vertex::MeshVertex;

pub trait MeshyObject {
    fn get_shader_group(&self) -> ShaderGroup;
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
        MeshVertex::transform_verts(&mut self.vertices, delta)
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
}

impl MeshyObject for RawMesh {
    fn get_shader_group(&self) -> ShaderGroup {
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
