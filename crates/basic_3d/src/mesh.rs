use frosty_alloc::{AllocId, FrostyAllocatable};
use render::vertex::MeshVertex;

pub struct Mesh3d<I> {
    verts: Vec<MeshVertex>,
    indices: Vec<I>,
}

unsafe impl<I> FrostyAllocatable for Mesh3d<I> {
    fn id() -> AllocId
    where
        Self: Sized,
    {
        AllocId::new(17)
    }
}
