mod access;
mod allocator;
mod chunk;

pub use access::*;
pub use allocator::Allocator;
use hashbrown::HashMap;

pub unsafe trait FrostyAllocatable {}
