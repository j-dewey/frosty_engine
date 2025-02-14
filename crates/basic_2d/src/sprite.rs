use engine_core::alloc::{AllocId, FrostyAllocatable};
use render::mesh::Mesh;

pub struct Sprite {
    mesh: Mesh,
}

unsafe impl FrostyAllocatable for Sprite {
    fn id() -> AllocId
    where
        Self: Sized,
    {
        AllocId::new(1000)
    }
}
