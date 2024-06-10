use crate::FrostyAllocatable;

// This determines when a system will re-query
pub enum SystemQuerySchedule {
    // Everytime the allocator has an object added to it
    OnEntitySpawn,
    // Each frame update
    Update,
}

pub trait System {
    type Input: FrostyAllocatable;
    fn query_type() -> SystemQuerySchedule {
        SystemQuerySchedule::OnEntitySpawn
    }
    fn query(&mut self, objs: &[&Self::Input]);
    fn update(&mut self, objs: &[&mut Self::Input]);
}
