mod access;
mod allocator;
mod chunk;
mod system;

pub use access::*;
pub use allocator::Allocator;
use hashbrown::HashMap;
pub use system::*;

type Index = usize;

pub unsafe trait FrostyAllocatable {}

// An interface to easily let [System]s
// interact with an [Allocator]
pub struct FrostyAllocInterface {
    alloc: Allocator,
    named_entries: HashMap<AllocName, Index>,
    sys_interops: HashMap,
}

impl FrostyAllocInterface {}
