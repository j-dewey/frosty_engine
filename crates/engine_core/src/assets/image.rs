use std::io::BufReader;

use image::{GenericImageView, ImageFormat};
use render::{
    scheduled_pipeline::{ScheduledBindGroup, ScheduledBindGroupType, ShaderLabel},
    texture::Texture,
    window_state::GPUBindings,
    winit::dpi::PhysicalSize,
};

// Load an image file. Can be PNG or JPEG
// returns (bytes, (width, height))
pub fn load_image(name: &str) -> (Vec<u8>, (u32, u32)) {
    // access file
    let path = name; //get_qualified_path(&("textures/".to_owned() + name));
    let file_handle = std::fs::File::open(path).unwrap();

    // find the format
    let file_tag = name
        .split(".")
        .last()
        .expect("Texture file is missing format tag!");
    let format = match file_tag {
        "png" => ImageFormat::Png,
        "jpg" | "jpeg" => ImageFormat::Jpeg,
        _ => panic!("Passed in a texture in an unacceptable file format!"),
    };

    let img = image::load(BufReader::new(file_handle), format).expect("Failed to read image data");
    let img_data = img.to_rgba8();
    (img_data.into_vec(), img.dimensions())
}

pub fn load_texture_array<'a>(
    label: ShaderLabel,
    files_to_load: Vec<String>,
    gpu: &GPUBindings,
) -> ScheduledBindGroup<'a> {
    // Load textures
    let textures: Vec<Texture> = files_to_load
        .iter()
        .map(|file| {
            let (img_data, (img_width, img_height)) = load_image(file);
            let texture = Texture::new(file, PhysicalSize::new(img_width, img_height), &gpu.device);
            texture.draw_image(&img_data[..], &gpu.queue);
            texture
        })
        .collect();

    // Move them into bind group
    ScheduledBindGroup {
        label,
        form: ScheduledBindGroupType::ReadOnlyTextureArray(textures),
    }
}
