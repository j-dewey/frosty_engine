use std::io;
use std::mem::MaybeUninit;
use std::sync::{Arc, Mutex};

use crate::system::SystemInterface;

pub(crate) enum ThreadMode {
    Query,
    Update,
}

struct ThreadState<'a> {
    run_thread: bool,
    thread_finished: bool,
    mode: ThreadMode,
    system: Option<&'a mut dyn SystemInterface>,
}

impl ThreadState<'_> {
    fn new() -> Self {
        Self {
            run_thread: false,
            thread_finished: true,
            mode: ThreadMode::Query,
            system: None,
        }
    }
}

// Threads which can run systems
// Used for thread pools in [App]
pub(crate) struct SystemThread<'a> {
    state: Arc<Mutex<ThreadState<'a>>>,
    // uninit only during instantiation
    thread: MaybeUninit<std::thread::JoinHandle<()>>,
}

impl SystemThread<'static> {
    pub(crate) fn new_unset() -> Self {
        let state = Arc::new(Mutex::new(ThreadState::new()));
        Self {
            state,
            thread: MaybeUninit::zeroed(),
        }
    }

    pub(crate) fn set_thread(&mut self, thread_builder: std::thread::Builder) -> io::Result<()> {
        let state_ptr = self.state.clone();

        let thread = thread_builder.spawn(move || loop {
            // 1) run the thing
            // 2) update the return value
            // 3) wait for new System
            if !state_ptr.lock().unwrap().run_thread {
                continue;
            }
        })?;

        self.thread.write(thread);
        io::Result::Ok(())
    }
}
