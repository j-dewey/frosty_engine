//
// functions for useful asset management
//

use std::io::BufReader;

use image::{GenericImageView, ImageFormat};

pub mod obj;

// Load an image file. Can be PNG or JPEG
// returns (bytes, (width, height))
pub fn load_image(name: &str) -> (Vec<u8>, (u32, u32)) {
    // access file
    let path = name; //get_qualified_path(&("textures/".to_owned() + name));
    let file_handle = std::fs::File::open(path).unwrap();

    // find format
    // TODO:
    //      This fails if a '.' appears elsewhere in the file path
    let file_tag = name
        .split(".")
        .skip(1)
        .next()
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
