use std::io::Write;

use frosty_alloc::debug::DebugOutter;
use render::{
    wgpu,
    window_state::WindowState,
    winit::{
        application::ApplicationHandler,
        dpi::PhysicalSize,
        event::WindowEvent,
        event_loop::{ActiveEventLoop, EventLoop},
        window::{Window, WindowId},
    },
};

use crate::{
    input,
    render_core::DynamicRenderPipeline,
    thread::{AppAlert, ThreadPool},
    Scene, SceneBuilder,
};

pub struct App {
    thread_pool: ThreadPool,
}

impl App {
    pub fn new() -> Self {
        let thread_pool = ThreadPool::new().expect("Failed to load threads");
        Self { thread_pool }
    }

    // The window can only be created once the event loop is running,
    // so the scene is built from a callback that receives the window size
    pub fn run(
        self,
        initial_scene: impl FnOnce(PhysicalSize<u32>) -> SceneBuilder,
        event_loop: EventLoop<()>,
    ) {
        self.run_handler(initial_scene, event_loop, None);
    }

    pub fn run_with_log(
        self,
        initial_scene: impl FnOnce(PhysicalSize<u32>) -> SceneBuilder,
        event_loop: EventLoop<()>,
        log_path: &str,
    ) {
        self.run_handler(initial_scene, event_loop, Some(log_path.to_owned()));
    }

    fn run_handler(
        self,
        initial_scene: impl FnOnce(PhysicalSize<u32>) -> SceneBuilder,
        event_loop: EventLoop<()>,
        log_path: Option<String>,
    ) {
        let mut handler = AppHandler {
            thread_pool: self.thread_pool,
            initial_scene: Some(initial_scene),
            log_path,
            state: None,
        };
        event_loop
            .run_app(&mut handler)
            .expect("Error encountered during main loop");
    }
}

// Drives an App from winit's event loop
struct AppHandler<F> {
    thread_pool: ThreadPool,
    // taken when the window is created
    initial_scene: Option<F>,
    log_path: Option<String>,
    // filled in once the window exists
    state: Option<(WindowState<'static>, Scene)>,
}

fn render(ws: &mut WindowState, pipeline: &mut DynamicRenderPipeline, elwt: &ActiveEventLoop) {
    if let Some((encoder, view, output)) = match ws.prep_render() {
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
        pipeline
            .draw(view, encoder, output, ws)
            .expect("Error during rendering :(");
    }
}

impl<F> ApplicationHandler for AppHandler<F>
where
    F: FnOnce(PhysicalSize<u32>) -> SceneBuilder,
{
    fn resumed(&mut self, elwt: &ActiveEventLoop) {
        // resumed can fire more than once, only set up the first time
        let Some(initial_scene) = self.initial_scene.take() else {
            return;
        };

        // The window lives for the rest of the program, so leak it
        // to let WindowState borrow it
        let window: &'static Window = Box::leak(Box::new(
            elwt.create_window(Window::default_attributes())
                .expect("Failed to create window"),
        ));
        let ws = pollster::block_on(WindowState::new(window));

        // init only fails if input is already init,
        // so no further work needed on it
        #[allow(unused_must_use)]
        unsafe {
            input::init_input(ws.window.inner_size());
            input::register_general_actions();
        }

        let initial_scene = initial_scene(ws.window.inner_size());
        if let Some(log_path) = &self.log_path {
            let mut out_file = std::fs::File::create(log_path).expect("failed to open log file");
            initial_scene.dump_data(&mut out_file);
            out_file.flush().unwrap();
        }

        let scene = initial_scene.build(&ws);
        self.state = Some((ws, scene));
    }

    fn window_event(&mut self, elwt: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        let Some((ws, scene)) = &mut self.state else {
            return;
        };
        if !(window_id == ws.window.id() && unsafe { !input::receive_window_input(&event) }) {
            // event already registered or belongs to a different
            // window, so just skip it
            return;
        }
        match event {
            WindowEvent::CloseRequested => elwt.exit(),
            WindowEvent::RedrawRequested => {
                let (alloc, schedule, pipeline) = scene.get_mutable_parts();
                match self.thread_pool.follow_schedule(schedule, alloc) {
                    AppAlert::None => {}
                    AppAlert::CloseApp => elwt.exit(),
                }

                render(ws, pipeline, elwt);

                #[allow(unused_must_use)]
                unsafe {
                    input::flush_frame_updates()
                };
                ws.window.request_redraw();
            }
            _ => {}
        }
    }
}

pub struct WindowlessApp {
    thread_pool: ThreadPool,
}

impl WindowlessApp {
    pub fn new() -> Self {
        let thread_pool = ThreadPool::new().expect("Failed to load threads");

        // init only fails if input is already init, so further work needed on it
        unsafe {
            #[allow(unused_must_use)]
            input::init_input(render::winit::dpi::PhysicalSize {
                width: 0,
                height: 0,
            });
        }

        Self { thread_pool }
    }

    pub fn run(self, initial_scene: SceneBuilder) {
        let (mut alloc, mut schedule, _) = initial_scene.dissolve();
        let mut done = false;
        while !done {
            match self.thread_pool.follow_schedule(&mut schedule, &mut alloc) {
                AppAlert::None => {}
                AppAlert::CloseApp => done = true,
            }
        }
    }
}
