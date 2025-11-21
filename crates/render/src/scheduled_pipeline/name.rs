use std::{hash::Hash, sync::Arc};

pub type Index = usize;

// Buffer / BindGroup / Texture Name
// This is a name to allow for accessing specific Buffers
// and BindGroups in ScheduledRequests and the ScheduledPipeline. Also
// for setting up Textures in ScheduledDescription.
//
//
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum ShaderLabel {
    Static(&'static str),
    Dynamic(Arc<str>),
}

impl ShaderLabel {
    pub const fn from_static_str(label: &'static str) -> Self {
        Self::Static(label)
    }

    pub fn label<'a>(&'a self) -> &'a str {
        match self {
            Self::Static(s) => s,
            Self::Dynamic(refer) => refer,
        }
    }
}

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
