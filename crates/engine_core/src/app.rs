use render::{
    wgpu,
    window_state::WindowState,
    winit::{
        event::{Event, WindowEvent},
        event_loop::{EventLoop, EventLoopWindowTarget},
        window::Window,
    },
};

use crate::{
    input,
    render_core::DynamicRenderPipeline,
    thread::{AppAlert, ThreadPool},
    SceneBuilder, Spawner,
};

pub struct App<'a> {
    thread_pool: ThreadPool,
    ws: WindowState<'a>,
}

impl<'a> App<'a> {
    pub fn new(window: &'a Window) -> Self {
        let thread_pool = ThreadPool::new().expect("Failed to load threads");
        let ws = pollster::block_on(WindowState::new(window));

        // init only fails if input is already init, so further work needed on it
        unsafe {
            #[allow(unused_must_use)]
            input::init_input(ws.window.inner_size());
        }

        Self { thread_pool, ws }
    }

    fn render(
        &mut self,
        pipeline: &mut DynamicRenderPipeline,
        alloc: &Spawner,
        elwt: &EventLoopWindowTarget<()>,
    ) {
        if let Some((encoder, view, output)) = match self.ws.prep_render() {
            Ok((view, encoder, output)) => Some((encoder, view, output)),
            // Reconfigure the surface if lost
            Err(wgpu::SurfaceError::Lost) => {
                self.ws.resize(self.ws.size);
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
                .draw(view, encoder, output, &mut self.ws)
                .expect("Error during rendering :(");
        }
    }

    pub fn run(mut self, initial_scene: SceneBuilder, event_loop: EventLoop<()>) {
        let mut scene = initial_scene.build(&self.ws);
        event_loop
            .run(move |event, elwt| {
                if let Event::WindowEvent { window_id, event } = event {
                    if !(window_id == self.ws.window.id()
                        && unsafe { !input::receive_window_input(&event) })
                    {
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

                            self.render(pipeline, alloc, elwt);

                            #[allow(unused_must_use)]
                            unsafe {
                                input::flush_frame_updates()
                            };
                            self.ws.window.request_redraw();
                        }
                        _ => {}
                    }
                }
            })
            .expect("Error encountered during main loop");
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
