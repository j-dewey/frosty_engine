//
// These are tests to ensure that asset loading works as intended
//

use engine_core::assets::AssetManager;
use render::{
    window_state::{GPUBindings, WindowState},
    winit::{event_loop::EventLoop, window::WindowBuilder},
};

#[test]
fn load_object_material_and_textures() {
    // Prepare gpu for texture loads
    let gpu = pollster::block_on(GPUBindings::new(None));

    // This is the actual test stuff
    let mut manager = AssetManager::new(String::from("../../res/tests/asset_load/"))
        .add("table.obj")
        .expect("Failed to add .obj file type to pending asset list");
    manager.read(&gpu);
}
