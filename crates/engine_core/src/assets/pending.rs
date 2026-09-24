//
// This is a collection of components that will automatically
// convert from a file path to a real asset

use frosty_alloc::FrostyAllocatable;
use render::{mesh::Mesh, vertex::MeshVertex};

use crate::render_core::TextureLoader;

// Indicates that a component stores the path to a loadable asset file
pub trait AssetFile {
    type Output: FrostyAllocatable;

    fn get_file(&self) -> &str;
}

// A .obj that will automatically convert to a Mesh<MeshVertex>
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MeshFile(pub String);
unsafe impl FrostyAllocatable for MeshFile {}
impl AssetFile for MeshFile {
    type Output = Mesh<MeshVertex>;
    fn get_file(&self) -> &str {
        &self.0
    }
}

// Currently only supports .png and .jpg
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageFile(pub String);
unsafe impl FrostyAllocatable for ImageFile {}
impl AssetFile for ImageFile {
    type Output = TextureLoader;
    fn get_file(&self) -> &str {
        &self.0
    }
}
