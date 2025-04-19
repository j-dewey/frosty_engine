pub type Index = usize;

// Buffer / BindGroup / Texture Name
// This is a name to allow for accessing specific Buffers
// and BindGroups in ScheduledRequests and the ScheduledPipeline. Also
// for setting up Textures in ScheduledDescription.
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub struct ShaderLabel(pub &'static str);

// Uniforms and Textures are coneceptually different
// from eachother, but both communicate wit the GPU
// through BindGroups. This index allows for easily
// accessing BindGroups in the required order even
// though they are placed in seperate caches.
#[derive(Clone, Copy, Debug)]
pub enum BindGroupIndex {
    Texture(Index),
    Uniform(Index),
}
