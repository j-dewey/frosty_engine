use render::shader::ShaderGroup;
use render::wgpu::BindGroup;

mod buffer_pointer;
pub use buffer_pointer::BufferPointer;
pub mod layout;
mod shader_node;

struct Statics<'a> {
    mesh: ShaderGroup<'a>,
    bind_groups: Vec<BindGroup>,
}

// Control the rendering pipeline in all stages:
// - Collecting mesh data from allocator
// - Collecting and caching bind groups
pub struct DynamicRenderPipeline {}

impl DynamicRenderPipeline {
    pub fn new(details: &layout::DRPLayout) -> Self {
        Self {}
    }
}
