use frosty_alloc::FrostyAllocatable;

/*
 * A system is composed of 3 parts:
 * 1) An [Interop] object
 * 2) A {System} object
 * 3) A {SystemInterface} object
 *
 * An [Interop] is a struct which a {System}
 * is able to read from and edit. An (Entity) consists
 * of multiple [Interop]s so that multiple {System}s can
 * interact with it. [Interop]s are seperate from the
 * {System}s which define them, so one [Interop] can
 * be altered by multiple {System}s.
 *
 * A {System} is an object which defines an [Interop],
 * a query(), and an update(). A query() reads a list of
 * [Interop]s loaded, then alters the state of the {System}.
 * An update() reads a list of [Interop]s loaded and alters
 * their state or the {System}'s.
 *
 * A {SystemInterface} is a wrapper object which allows the
 * [AllocInterface] to store the corresponding {System}. This
 * is needed due to how generics work. The {SystemInterface}
 * can be either a unique object or the same as the {System}
 */

type PerSecond = u32;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct SystemId(u64);

// This determines when a system will re-query
pub enum SystemQuerySchedule {
    // Everytime the allocator has an object added to it
    OnEntitySpawn,
    // Each frame update
    Update,
}

pub enum SystemUpdateSchedule {
    // updates with each frame update
    Variable,
    // updates a specific number of times each second
    Fixed(PerSecond),
}

pub trait System {
    type Interop: FrostyAllocatable;
    fn query(&mut self, objs: &[&Self::Interop]);
    fn update(&mut self, objs: &[&mut Self::Interop]);
}

pub trait SystemInterface {
    fn query_type() -> SystemQuerySchedule {
        SystemQuerySchedule::OnEntitySpawn
    }
    // rules for dependencies:
    //      Variable -> Needs to occur before a system can
    //                  update
    //      Fixed    -> Needs to pause before a system can
    //                  update
    fn dependencies() -> Vec<SystemId>;
    fn id() -> SystemId;
    fn query(&mut self, objs: &[&dyn FrostyAllocatable]);
    fn update(&mut self, objs: &[&dyn FrostyAllocatable]);
}
