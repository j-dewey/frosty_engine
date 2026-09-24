use frosty_alloc::FrostyAllocatable;
use render::scheduled_pipeline::ShaderLabel;

// Textures take up a *lot* of space, so only want them loaded
// in heap memory while they're being sent to gpu. Filenames
// generally aren't that big, so they can float around for a while.
//
// These objects really don't need to stay around after though,
// so they should get deleted pretty quickly
pub struct TextureLoader {
    pub label: ShaderLabel,
    pub width: u32,
    pub height: u32,
    pub pixel_data: Vec<u8>,
}

unsafe impl FrostyAllocatable for TextureLoader {}
