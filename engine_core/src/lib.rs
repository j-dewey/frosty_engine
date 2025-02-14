#![feature(unsize)]

mod concur;
mod schedule;
mod thread;

#[cfg(not(feature = "no-app"))]
mod app;
pub use app::App;

mod entity;
pub use entity::Entity;
pub mod query;
mod scene;
pub use scene::Scene;
mod spawner;
pub use spawner::Spawner;
pub mod render_core;

pub mod input;

#[cfg(not(feature = "no-system"))]
mod system;
pub use system::{System, SystemId, SystemInterface, SystemQuerySchedule, SystemUpdateSchedule};

// The thread which runs all systems and switches
// between loop sections
pub const MASTER_THREAD: u32 = 0;
