#![feature(iter_advance_by)]
pub mod window_state;

pub mod color;
pub mod gui_mesh;
pub mod mesh;
pub mod shader;
pub mod texture;
pub mod uniform;
pub mod vertex;

pub mod scheduled_pipeline;

// re exports
pub use wgpu;
pub use winit;

pub const QUAD_INDEX_ORDER: [u32; 6] = [0, 2, 1, 1, 2, 3];
