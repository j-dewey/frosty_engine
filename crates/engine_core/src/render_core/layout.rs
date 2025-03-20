use render::{texture::Texture, wgpu};

use crate::query::DynQuery;

use super::GivesBindGroup;

// This reflects the wgpu::BindingResource
//
pub enum ShaderResourceType {
    Buffer,
    BufferArray { len: u32 },
    Sampler,
    SamplerArray { len: u32 },
    TextureView,
    TextureViewArray { len: u32 },
}

pub struct ShaderNodeLayout<'a> {
    // shader file
    pub source: &'static str,
    // description of vertexes in mesh
    pub vertex_desc: wgpu::VertexBufferLayout<'a>,
    // a query to all bindgroups for this shader in Allocator
    pub bind_groups: DynQuery<dyn GivesBindGroup + 'static>,
    // details about the textures rendered to
    // index in vec corresponds to
    pub out_textures: Option<Vec<wgpu::TextureDescriptor<'a>>>,
    pub depth_stencil: Option<wgpu::DepthStencilState>,
    pub depth_buffer: Option<Texture>,
}
