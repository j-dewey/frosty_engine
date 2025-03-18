use render::{
    wgpu,
    window_state::WindowState,
    winit::{
        event::{Event, WindowEvent},
        event_loop::{EventLoop, EventLoopWindowTarget},
        window::{Window, WindowBuilder},
    },
};

use crate::{input, scene::Scene, thread::ThreadPool};

pub struct App<'a> {
    thread_pool: ThreadPool,
    scene: Scene,
    ws: WindowState<'a>,
}

impl<'a> App<'a> {
    pub fn new(scene: Scene, window: &'a Window) -> Self {
        let thread_pool = ThreadPool::new().expect("Failed to load threads");
        let ws = pollster::block_on(WindowState::new(window));

        // init only fails if input is already init, so further work needed on it
        unsafe {
            #[allow(unused_must_use)]
            input::init_input(ws.window.inner_size());
        }

        Self {
            thread_pool,
            scene,
            ws,
        }
    }

    pub fn register_components<F: FnOnce(App<'_>) -> App<'_>>(self, registration: F) -> Self {
        registration(self)
    }

    pub fn register_rendering(mut self) -> Self {
        todo!()
    }

    fn render(&mut self, elwt: &EventLoopWindowTarget<()>) {
        if let Some((mut encoder, view, output)) = match self.ws.prep_render() {
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
            self.ws.post_render(encoder, output);
        }
    }

    pub fn run(mut self, event_loop: EventLoop<()>) {
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
                            let (alloc, schedule) = self.scene.get_mutable_parts();
                            self.thread_pool.follow_schedule(schedule, alloc);

                            self.render(elwt);

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
