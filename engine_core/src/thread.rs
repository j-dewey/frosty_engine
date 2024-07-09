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
struct SystemThread<'a> {
    state: Arc<Mutex<ThreadState<'a>>>,
    // uninit only during instantiation
    thread: MaybeUninit<std::thread::JoinHandle<()>>,
}

impl SystemThread<'static> {
    fn new_unset() -> Self {
        let state = Arc::new(Mutex::new(ThreadState::new()));
        Self {
            state,
            thread: MaybeUninit::zeroed(),
        }
    }

    fn set_thread(&mut self, thread_builder: std::thread::Builder) -> io::Result<()> {
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

pub(crate) struct ThreadPool {
    threads: Vec<SystemThread<'static>>,
}

impl ThreadPool {
    pub(crate) fn new() -> io::Result<Self> {
        // TODO:
        //  find number of cores for maximum threading
        //  efficiency
        let thread_count = 4;
        let mut threads = Vec::with_capacity(thread_count);
        for thread in 0..thread_count {
            let thread_builder = std::thread::Builder::new();
            let t = SystemThread::new_unset();
            threads.push(t);
            threads[thread].set_thread(thread_builder)?;
        }
        io::Result::Ok(Self { threads })
    }
}
