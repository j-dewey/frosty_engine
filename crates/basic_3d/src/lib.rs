use ::render::{
    mesh::Mesh,
    vertex::{MeshVertex, ScreenQuadVertex},
};
use engine_core::{
    package::{register, Package},
    render_core::TextureLoader,
};

use crate::camera::Camera3d;

pub mod camera;
pub mod render;

pub const GENERAL_3D_PACKAGE: Package<4> = Package::new([
    // Common Scene Items
    // Required For General3dPipeline
    register::<Camera3d>(),
    register::<Mesh<MeshVertex>>(),
    register::<Mesh<ScreenQuadVertex>>(),
    register::<TextureLoader>(),
]);
