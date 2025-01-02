use cgmath::Point3;
use engine_core::{
    render_core::{layout::*, DynamicRenderPipeline},
    Spawner,
};
use frosty_alloc::FrostyAllocatable;
use render::{
    window_state::WindowState,
    winit::{event_loop::EventLoop, window::WindowBuilder},
};

mod snow_mesh;
use snow_mesh::*;

fn set_up_pipeline<'a>(win_width: u32, win_height: u32) -> DRPLayout<'a> {
    DRPLayout {
        nodes: vec![
            // shell textured snow
            ShaderNodeLayout {
                mesh_id: SnowMesh::id(),
                vertex_desc: SnowVertex::desc(),
                bind_groups: vec![
                    // camera
                    ShaderBindGroup {
                        entries: vec![ShaderBindGroupEntryLayout {
                            binding: 0,
                            resource_type: ShaderResourceType::Buffer,
                        }],
                    },
                ],
                out_textures: None,
                use_depth: true,
            },
        ],
    }
}

fn set_up_scene(ws: &WindowState) -> Spawner {
    let mut spawner = Spawner::new();
    spawner.register_component::<SnowMesh>();

    let snow = SnowMesh::new(10.0, 32, 0.1, Point3::new(0.0, 0.0, 0.0), ws);

    spawner
}

async fn run() {
    env_logger::init();
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let win_dims = window.inner_size();
    let ws = WindowState::new(&window).await;

    let spawner = set_up_scene(&ws);

    let render_pipeline_layout = set_up_pipeline(win_dims.width, win_dims.height);
    let renderer = DynamicRenderPipeline::new(&render_pipeline_layout);
}

pub(crate) fn main() {
    pollster::block_on(run());
}
