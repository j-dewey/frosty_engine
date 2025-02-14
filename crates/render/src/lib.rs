use hashbrown::HashMap;

pub mod window_state;

pub mod color;
pub mod gui_mesh;
pub mod material;
pub mod mesh;
pub mod post_process;
pub mod shader;
pub mod texture;
pub mod vertex;

// re exports
pub use wgpu;
pub use winit;

pub const QUAD_INDEX_ORDER: [u32; 6] = [0, 2, 1, 1, 2, 3];

pub struct SceneRenderInfo<'a> {
    pub shaders: HashMap<&'a str, shader::Shader>,
    pub bg_layouts: HashMap<&'a str, wgpu::BindGroupLayout>,
    pub mats: Vec<material::Material>,
    pub material_array: wgpu::BindGroup,
}
