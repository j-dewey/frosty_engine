// This crate has 3 main responsibilities:
//    1) Implement threading
//    2) Provide helpful utilities
//    3) Provide interfaces and maintain book keeping for low level things
//
// In regards to the 3rd responsibility, the entire ECS and DRP systems
// could be considered interfaces and book keeping. [Spawner] is a thin
// wrapper for [SystemAllocator] and [DynamicRenderPipeline] is a thin
// wrapper for [ScheduledPipeline]. To prevent these objects from becoming
// bloated, there is some functionality lost during this wrapping process.
// Simply exposing the underlying object just feels wrong.
//
// This file offers a trait to allow this exposure for select objects.

use frosty_alloc::Allocator;
use render::scheduled_pipeline::ScheduledPipeline;

use crate::{render_core::DynamicRenderPipeline, Spawner};

pub trait Facade {
    type Underlying;
    fn get_basic(&self) -> &Self::Underlying;
}

impl Facade for Spawner {
    type Underlying = Allocator;
    fn get_basic(&self) -> &Self::Underlying {
        &self.alloc
    }
}

impl Facade for DynamicRenderPipeline {
    type Underlying = ScheduledPipeline;
    fn get_basic(&self) -> &Self::Underlying {
        &self.pipeline
    }
}
