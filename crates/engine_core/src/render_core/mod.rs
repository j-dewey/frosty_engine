use frosty_alloc::{AllocId, FrostyAllocatable};
use layout::ShaderNodeLayout;
use render::mesh::MeshyObject;
use render::shader::ShaderGroup;
use render::texture::Texture;
use render::vertex::Vertex;
use render::wgpu::{self, BindGroup};
use render::window_state::WindowState;

pub mod layout;
mod shader_node;
pub use shader_node::ShaderNode;

use crate::query::Query;
use crate::{Spawner, MASTER_THREAD};

pub trait GivesBindGroup: FrostyAllocatable {
    fn get_bind_group_layout(&self, ws: &WindowState) -> wgpu::BindGroupLayout;
    fn get_bind_group(&self, ws: &WindowState) -> wgpu::BindGroup;
}

struct Statics<'a> {
    mesh: ShaderGroup<'a>,
    bind_groups: Vec<BindGroup>,
}

// These functions are just wrappers for ShaderNode<M>.draw()
// to hide M
type RenderFn = Box<dyn Fn(Query<u8>, &mut WindowState) + 'static>;

// Control the rendering pipeline in all stages:
// - Collecting mesh data from allocator
// - Collecting and caching bind groups
pub struct DynamicRenderPipeline {
    render_fns: Vec<(RenderFn, AllocId)>,
    // Textures that are shared across ShaderNodes
    // This needs to be implemented
    texture_cache: Vec<Texture>,
}

impl DynamicRenderPipeline {
    // This is helpful for quickly prototyping without worrying about how things
    // are rendered
    pub fn new_empty() -> Self {
        Self {
            render_fns: Vec::new(),
            texture_cache: Vec::new(),
        }
    }

    pub fn register_shader<'a, M: MeshyObject + FrostyAllocatable + 'static, V: Vertex>(
        &mut self,
        layout: ShaderNodeLayout<'a>,
        ws: &WindowState,
        spawner: &Spawner,
    ) {
        // TODO:
        //      Make shared textures
        let shader: ShaderNode<M> = ShaderNode::new::<V>(layout, ws, spawner);
        self.render_fns.push((shader.init_render_fn(), M::id()));
    }

    // Draws with shaders based on registration order
    // TODO:
    //      Allow for shaders dependant on other shaders
    //      while adding concurrency
    pub fn draw(&mut self, spawner: &Spawner, ws: &mut WindowState) {
        for (render_fn, id) in &mut self.render_fns {
            let query = spawner
                .get_dissolved_query(*id, MASTER_THREAD)
                .expect("Failed to find Mesh Query");
            (render_fn)(query, ws);
        }
    }
}
