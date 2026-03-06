#![feature(unsize)]
#![feature(impl_trait_in_bindings)]
#![feature(iter_array_chunks)]
#![feature(try_trait_v2)]
#![feature(const_type_id)]

mod concur;
mod schedule;
mod thread;

#[cfg(not(feature = "no-app"))]
pub mod app;
pub use app::App;

use frosty_alloc::AllocGroup;
pub mod assets;
pub mod debug;
pub mod facade;
pub mod package;
pub mod query;
mod scene;
pub use scene::{Scene, SceneBuilder};
mod spawner;
pub use spawner::Spawner;
pub mod render_core;

pub mod input;

#[cfg(not(feature = "no-system"))]
pub mod system;

// The thread which runs all systems and switches
// between loop sections
pub const MASTER_THREAD: u32 = 0;

pub type Entity = AllocGroup;
