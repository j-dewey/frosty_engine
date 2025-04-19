use frosty_alloc::FrostyAllocatable;
use hashbrown::HashMap;
use render::mesh::MeshyObject;
use render::scheduled_pipeline::{ScheduledPipeline, ShaderLabel};
use render::vertex::Vertex;
use render::wgpu;
use render::window_state::WindowState;

pub mod layout;
mod shader_node;
pub use shader_node::{DataCollector, DynamicNode, DynamicNodeDefinition};

use crate::{Spawner, MASTER_THREAD};

pub trait GivesBindGroup: FrostyAllocatable {
    fn get_bind_group_layout(ws: &WindowState) -> wgpu::BindGroupLayout
    where
        Self: Sized;
    fn get_uniform_data(&self) -> Box<[u8]>;
}

pub struct DynamicRenderPipelineDescriptor<'a> {
    collectors: &'a [DataCollector],
}

// Control the rendering pipeline in all stages:
// - Collecting mesh data from allocator
// - Collecting and caching bind groups
pub struct DynamicRenderPipeline {
    data_collectors: Vec<DataCollector>, // this collects shader data
    pipeline: ScheduledPipeline,         // this stores all shader data
    node_names: HashMap<ShaderLabel, usize>, // maps node name to index. order based on pipeline definition
}

impl DynamicRenderPipeline {
    pub fn new(pipeline: ScheduledPipeline, node_order: Vec<ShaderLabel>) -> Self {
        let mut node_names = HashMap::with_capacity(node_order.len());
        for (indx, name) in node_order.iter().enumerate() {
            node_names.insert(*name, indx);
        }
        Self {
            data_collectors: Vec::new(),
            pipeline,
            node_names,
        }
    }

    pub fn register_shader<'a, M: MeshyObject + FrostyAllocatable + 'static, V: Vertex>(
        mut self,
        def: DynamicNodeDefinition<M>,
        ws: &WindowState,
        alloc: &Spawner,
    ) -> Self {
        // 1) TODO: verify it is compatible with the current pipeline
        // 2) set up queries
        // 3) add data collector to array

        let mesh_query = alloc
            .get_query::<M>(MASTER_THREAD)
            .expect("Failed to register Mesh object to Spawner");

        let data_collector = DynamicNode {
            meshes: mesh_query,
            bind_groups: def.bind_groups,
            buffer_label: def.node,
        }
        .to_collection_function();

        self.data_collectors.push(data_collector);
        self
    }

    // Draws with shaders based on registration order
    // TODO:
    //      Allow for shaders dependant on other shaders
    //      while adding concurrency
    pub fn draw(&mut self, ws: &mut WindowState) {
        for collector in &mut self.data_collectors {
            (collector)(&mut self.pipeline, ws);
        }
    }
}
