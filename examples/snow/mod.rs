use basic_3d::camera::{Camera3d, Projection};
use cgmath::{Deg, Point3};
use engine_core::{
    input,
    query::DynQuery,
    render_core::{layout::*, DynamicRenderPipeline, GivesBindGroup},
    Spawner, MASTER_THREAD,
};
use render::{
    texture::Texture,
    vertex::Vertex,
    wgpu,
    window_state::WindowState,
    winit::{
        self,
        event::{Event, WindowEvent},
        event_loop::EventLoop,
        window::WindowBuilder,
    },
};

mod snow_mesh;
use snow_mesh::*;
mod snow_details;
use snow_details::SnowDetails;

fn set_up_pipeline<'a>(
    win_width: u32,
    win_height: u32,
    spawner: &Spawner,
    ws: &WindowState,
) -> DynamicRenderPipeline {
    let mut pipeline = DynamicRenderPipeline::new_empty();
    // find bind groups
    let mut snow_bind_groups: DynQuery<dyn GivesBindGroup> = DynQuery::new_empty();
    snow_bind_groups.push(
        &spawner
            .get_query::<Camera3d>(MASTER_THREAD)
            .expect("Unable to find Camera3D in Spawner")
            .into_iter()
            .next_handle()
            .expect("No Camera3D allocated"),
    );
    snow_bind_groups.push(
        &spawner
            .get_query::<SnowDetails>(MASTER_THREAD)
            .expect("Unable to find SnowDetails in Spawner")
            .into_iter()
            .next_handle()
            .expect("No SnowDetails allocated"),
    );
    let snow_shader = ShaderNodeLayout {
        source: include_str!("snow.wgsl"),
        vertex_desc: SnowVertex::desc(),
        bind_groups: snow_bind_groups,
        out_textures: None,
        depth_stencil: Some(wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }),
        depth_buffer: Some(Texture::new_depth_non_filter("depth", ws.size, &ws.device)),
    };
    pipeline.register_shader::<SnowMesh, SnowVertex>(snow_shader, ws, spawner)
}

fn set_up_scene(ws: &WindowState) -> Spawner {
    let mut spawner = Spawner::new();
    spawner.register_component::<Camera3d>();
    spawner.register_component::<SnowMesh>();
    spawner.register_component::<SnowDetails>();

    let win_size = ws.window.inner_size();
    spawner
        .spawn_obj(Camera3d::new(
            [0.0, 0.0, 0.0],
            Deg(0.0),
            Deg(0.0),
            Projection::new(win_size.width, win_size.height, Deg(90.0), 100.0, 0.01),
        ))
        .expect("Failed to register Camera3D to Spawner");

    spawner
        .spawn_obj(SnowMesh::new(10.0, 32, 0.1, Point3::new(0.0, 0.0, 0.0), ws))
        .expect("Failed to register SnowMesh to Spawner");
    spawner
        .spawn_obj(SnowDetails::new(
            128,
            128,
            1.0 / 16.0,
            [1.0, 1.0, 1.0],
            [0.0, 0.0, 0.0],
            1.0,
        ))
        .expect("Failed to register SnowDetails to Spawner");

    spawner
}

async fn set_up<'a>(
    window: &'a winit::window::Window,
) -> (WindowState<'a>, Spawner, DynamicRenderPipeline) {
    env_logger::init();
    let win_dims = window.inner_size();
    let ws = WindowState::new(&window).await;
    unsafe {
        input::init_input(ws.window.inner_size()).expect("INPUT_HANDLER init previously");
        input::register_general_actions().expect("INPUT_HANDLER failed to init");
    }

    let spawner = set_up_scene(&ws);

    let render_pipeline = set_up_pipeline(win_dims.width, win_dims.height, &spawner, &ws);
    (ws, spawner, render_pipeline)
}

async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();
    let (ws, spawner, drp) = set_up(&window).await;
    /*
    event_loop.run(move |event, elwt| {
        if let Event::WindowEvent { window_id, event } = event {
            if window_id == ws.window.id() && !input.recieve_window_input(&event) {
                match event {
                    WindowEvent::CloseRequested => elwt.exit(),
                    WindowEvent::RedrawRequested => {
                        if let Some((mut encoder, view, output)) = match ws.prep_render() {
                            Ok((view, encoder, output)) => Some((encoder, view, output)),
                            // Reconfigure the surface if lost
                            Err(wgpu::SurfaceError::Lost) => {
                                ws.resize(ws.size);
                                None
                            }
                            // The system is out of memory, we should probably quit
                            Err(wgpu::SurfaceError::OutOfMemory) => {
                                elwt.exit();
                                None
                            }
                            // All other errors (Outdated, Timeout) should be resolved by the next frame
                            Err(e) => {
                                eprintln!("{:?}", e);
                                None
                            }
                        } {
                            //scene.render(&ws, &mut encoder, &view);
                            (scene.render)(&ent_table, &ws, &ri, &mut encoder, &view);
                            ws.post_render(encoder, output);
                        }
                        if let Some(new_scene) = scene.to_load {
                            scene = (new_scene)(&mut ent_table, &mut ri, &ws);
                        }

                        input.flush_frame_updates();
                        timer.tick();
                        ws.window.request_redraw();
                    }
                    _ => {}
                }
            }
        }
    });*/
}

pub(crate) fn main() {
    pollster::block_on(run());
}
