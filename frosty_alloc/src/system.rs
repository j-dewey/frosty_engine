use crate::FrostyAllocatable;

// This determines when a system will re-query
pub enum SystemQuerySchedule {
    // Everytime the allocator has an object added to it
    OnEntitySpawn,
    // Each frame update
    Update,
}

/*
 * A system is composed of 3 parts:
 * 1) An [Interop] object
 * 2) A {System} object
 * 3) A {SystemInterface} object
 *
 *
 */

pub trait System {
    type Interop: FrostyAllocatable;
    fn query(&mut self, objs: &[&Self::Interop]);
    fn update(&mut self, objs: &[&mut Self::Interop]);
}

pub trait SystemInterface {
    fn query_type() -> SystemQuerySchedule {
        SystemQuerySchedule::OnEntitySpawn
    }
    fn query(&mut self, objs: &[&dyn FrostyAllocatable]);
    fn update(&mut self, objs: &[&dyn FrostyAllocatable]);
}
