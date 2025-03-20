#![feature(unsize)]
#![feature(impl_trait_in_bindings)]

mod concur;
mod schedule;
mod thread;

#[cfg(not(feature = "no-app"))]
pub mod app;
pub use app::App;

mod entity;
pub use entity::Entity;
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
