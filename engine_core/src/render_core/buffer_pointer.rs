use std::sync::Arc;

use render::wgpu;

// Buffer not implementing Copy means anything
// holding them cannot be spawned in Spawner.
// A BufferPointer allows for safely accessing
// and copying Buffer data
#[derive(Clone)]
pub struct BufferPointer {
    buffer: Arc<wgpu::Buffer>,
}
