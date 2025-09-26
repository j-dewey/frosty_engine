// Some helpful debugging systems

use std::{fmt::Debug, marker::PhantomData};

use frosty_alloc::FrostyAllocatable;

use crate::{
    query::Query,
    system::{System, UpdateResult},
};

// Print the value for some component which implements
// Debug each frame
pub struct DisplayingSystem<C>
where
    C: FrostyAllocatable + Debug,
{
    _pd: PhantomData<C>,
}

impl<C> System for DisplayingSystem<C>
where
    C: FrostyAllocatable + Debug,
{
    type Interop = C;

    fn update(&self, mut objs: Query<Self::Interop>, thread: u32) -> UpdateResult {
        while let Some(obj) = objs.next(thread) {
            println!("{:?}", obj.as_ref());
        }
        UpdateResult::Skip
    }
}
