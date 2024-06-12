use super::texture::Texture;

pub struct Material {
    texture: Texture,
    /*
     * other pbr things
     */
}

pub struct MaterialUniform {
    /*
     */
}

impl Material {
    pub fn from_text(texture: Texture) -> Self {
        Self { texture }
    }

    pub fn get_text(&self) -> &Texture {
        &self.texture
    }
}
