use std::{hash::Hash, sync::Arc};

pub type Index = usize;

// Buffer / BindGroup / Texture Name
// This is a name to allow for accessing specific Buffers
// and BindGroups in ScheduledRequests and the ScheduledPipeline. Also
// for setting up Textures in ScheduledDescription.
//
//
#[derive(Clone, Debug, Hash)]
pub enum ShaderLabel {
    Static(&'static str),
    Dynamic(Arc<String>),
}

impl PartialEq for ShaderLabel {
    fn eq(&self, other: &Self) -> bool {
        self.label() == other.label()
    }
}

impl Eq for ShaderLabel {}

// Here are some premade common labels

// For testing purposes, sets texture to the default one
pub const DEFAULT_MATERIAL_LABEL: ShaderLabel = ShaderLabel::from_static_str("default-material");
// Material is loaded at runtime, so label will be decided later
pub const PENDING_MATERIAL_LABEL: ShaderLabel = ShaderLabel::from_static_str("pending-material");
pub const PRESCREEN_RENDER_TARGET_TEXTURES: ShaderLabel =
    ShaderLabel::from_static_str("prescreen-render-target-textures");

impl ShaderLabel {
    pub const fn from_static_str(label: &'static str) -> Self {
        Self::Static(label)
    }

    pub fn from_string(label: String) -> Self {
        Self::Dynamic(Arc::new(label))
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
    MaterialList(Index),
}
