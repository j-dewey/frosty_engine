use wgpu::util::DeviceExt;

use super::{shader::ShaderGroup, QUAD_INDEX_ORDER};

#[repr(C)]
#[derive(Clone, Copy, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PostprocessVertex {
    position: [f32; 2],
}

impl PostprocessVertex {
    pub fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<PostprocessVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                // position
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0, // for @location(n) when defining struct in shader
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }

    pub fn generate_quad(device: &wgpu::Device) -> ShaderGroup {
        let verts = [
            Self {
                position: [-1.0, 1.0],
            },
            Self {
                position: [1.0, 1.0],
            },
            Self {
                position: [-1.0, -1.0],
            },
            Self {
                position: [1.0, -1.0],
            },
        ];

        let i_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&QUAD_INDEX_ORDER[..]),
            usage: wgpu::BufferUsages::INDEX,
        });

        let v_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("GUIMesh Vertex Buffer"),
            contents: bytemuck::cast_slice(&verts[..]),
            usage: wgpu::BufferUsages::VERTEX,
        });

        ShaderGroup::new_owned(v_buf, i_buf, 6)
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ScreenDetails {
    pub height: f32,
    pub width: f32,
    pub z_near: f32,
    pub z_far: f32,
}
