use frosty_alloc::AllocId;
use render::wgpu;

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

pub struct ShaderBindGroupEntryLayout {
    pub binding: u32,
    pub resource_type: ShaderResourceType,
}

pub struct ShaderBindGroup {
    pub entries: Vec<ShaderBindGroupEntryLayout>,
}

pub struct ShaderNodeLayout<'a> {
    pub mesh_id: AllocId,
    pub vertex_desc: wgpu::VertexBufferLayout<'a>,
    pub bind_groups: Vec<ShaderBindGroup>,
    // details about the textures rendered to
    // index in vec corresponds to
    pub out_textures: Option<Vec<wgpu::TextureDescriptor<'a>>>,
    pub use_depth: bool,
}

/*
* A layout defining a dynamic render pipeline
*/
pub struct DRPLayout<'a> {
    pub nodes: Vec<ShaderNodeLayout<'a>>,
}
