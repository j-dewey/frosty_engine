use std::marker::PhantomData;

use crate::query::{DynQuery, Query};
use crate::MASTER_THREAD;

use super::GivesBindGroup;
use frosty_alloc::FrostyAllocatable;
use render::mesh::MeshyObject;
use render::scheduled_pipeline::{BufferUpdate, NodeUpdateRequest, ScheduledPipeline, ShaderLabel};
use render::window_state::WindowState;

pub type DataCollector = Box<dyn FnMut(&mut ScheduledPipeline, &WindowState) -> () + 'static>;

pub struct DynamicNodeDefinition<M: MeshyObject + FrostyAllocatable> {
    pub bind_groups: DynQuery<dyn GivesBindGroup>, // handles must be manually added to this
    pub node: ShaderLabel,
    pub _pd: PhantomData<M>,
}

pub struct DynamicNode<M: MeshyObject + FrostyAllocatable> {
    pub(crate) meshes: Query<M>,
    pub(crate) bind_groups: DynQuery<dyn GivesBindGroup>,
    pub(crate) buffer_label: ShaderLabel,
}

impl<M: MeshyObject + FrostyAllocatable> DynamicNode<M> {
    // TODO:
    //      Only push updates to mutated data
    pub fn to_collection_function(mut self) -> DataCollector {
        Box::new(move |pipeline: &mut ScheduledPipeline, ws: &WindowState| {
            let mut updated_meshes = Vec::new();
            let mut updated_bind_groups = Vec::new();

            self.meshes.for_each(|m| {
                updated_meshes.push(BufferUpdate::Raw(
                    m.as_ref().get_verts() as *const [u8],
                    m.as_ref().get_indices() as *const [u8],
                ))
            });
            self.meshes.reset();

            self.bind_groups.for_each(|mut has_bg| {
                updated_bind_groups.push(Some(
                    has_bg
                        .get_access(MASTER_THREAD)
                        .expect("Cannot handle deallocated bind group currently")
                        .as_ref()
                        .get_uniform_data(),
                ))
            });
            self.bind_groups.reset();

            pipeline.update_node_caches(
                NodeUpdateRequest {
                    buffers: updated_meshes,
                    uniforms: updated_bind_groups,
                    mesh_label: self.buffer_label,
                },
                ws,
            );
        })
    }
}
