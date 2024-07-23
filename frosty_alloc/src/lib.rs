mod access;
mod allocator;
mod chunk;
mod query;

pub use access::*;
pub use allocator::Allocator;

pub unsafe trait FrostyAllocatable {
    fn id() -> AllocName
    where
        Self: Sized;
}
