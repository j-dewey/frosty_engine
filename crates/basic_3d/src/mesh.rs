use frosty_alloc::{AllocId, FrostyAllocatable};
use render::vertex::MeshVertex;

pub struct Mesh3d<I> {
    verts: Vec<MeshVertex>,
    indices: Vec<I>,
}

unsafe impl<I: 'static> FrostyAllocatable for Mesh3d<I> {}
