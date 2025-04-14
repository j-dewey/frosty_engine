use std::{any::TypeId, task::Poll};

use frosty_alloc::FrostyAllocatable;

use crate::query::Query;

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
pub struct SystemId(pub u64);

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

#[derive(PartialEq, Eq, Debug)]
pub enum UpdateResult {
    CloseApp,
    Spawn,
    Skip,
    PollingError,
}

impl From<Poll<UpdateResult>> for UpdateResult {
    fn from(value: Poll<UpdateResult>) -> Self {
        match value {
            Poll::Pending => Self::PollingError,
            Poll::Ready(res) => res,
        }
    }
}

pub trait System {
    type Interop: FrostyAllocatable;
    fn update(&self, objs: Query<Self::Interop>) -> UpdateResult;
}

/*
 * TODO: create a macro to auto implement this
 */
pub trait SystemInterface: Send + Sync + 'static {
    fn query_type() -> SystemQuerySchedule
    where
        Self: Sized,
    {
        SystemQuerySchedule::OnEntitySpawn
    }
    // rules for dependencies:
    //      Variable -> Needs to occur before a system can
    //                  update
    //      Fixed    -> Needs to pause before a system can
    //                  update
    fn dependencies() -> Vec<SystemId>
    where
        Self: Sized;
    fn id() -> SystemId
    where
        Self: Sized;
    fn alloc_id(&self) -> TypeId;
    // NOTE:
    //      currently takes Query by value, so each Interface.update() call
    //      owns the query and thus the system cannot be called across threads
    //      safely. This is fine for continuous systems, but it prevents discrete
    //      ones from being called concurrently
    fn start_update(&self, objs: Query<u8>) -> UpdateResult;
}
