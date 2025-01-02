use crate::query::Query;

use super::Statics;
use frosty_alloc::FrostyAllocatable;
use render::{mesh::MeshyObject, shader::Shader};

pub struct ShaderNode<'a, M: MeshyObject + FrostyAllocatable> {
    cache: Statics<'a>,
    meshes: Query<M>,
    shader: Shader,
}

impl<'a, M> ShaderNode<'a, M>
where
    M: MeshyObject + FrostyAllocatable,
{
    pub fn draw(&self) {
        let mut meshes = Vec::new();
        meshes.push(&self.cache.mesh);
        //meshes.extend(self.collection.get_all());

        //self.shader
        //    .render(&meshes[..], bind_groups, ri, bg_offset, encoder, view);
    }
}
