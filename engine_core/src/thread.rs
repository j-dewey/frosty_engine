use std::future::Future;
use std::io;
use std::mem::MaybeUninit;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::JoinHandle;

use frosty_alloc::Allocator;

use crate::query::Query;
use crate::schedule::{NextSystem, Schedule};
use crate::system::{SystemInterface, UpdateResult};

pub(crate) enum ThreadMode {
    Query,
    Update,
}

struct ThreadState {
    system: Receiver<(Box<dyn SystemInterface>, Query<u8>)>,
    output: Sender<UpdateResult>,
}

impl ThreadState {
    fn new() -> (
        Self,
        Sender<(Box<dyn SystemInterface>, Query<u8>)>,
        Receiver<UpdateResult>,
    ) {
        let (sys_send, sys_recv) = channel();
        let (out_send, out_recv) = channel();
        (
            Self {
                system: sys_recv,
                output: out_send,
            },
            sys_send,
            out_recv,
        )
    }
}

// Threads which can run systems
// Used for thread pools in [App]
struct SystemThread {
    // uninit only during instantiation
    thread: MaybeUninit<JoinHandle<()>>,
    system_sender: Sender<(Box<dyn SystemInterface>, Query<u8>)>,
    output_reciever: Receiver<UpdateResult>,
}

impl SystemThread {
    // create a new thread without any system set
    fn new_unset(
        sys_send: Sender<(Box<dyn SystemInterface>, Query<u8>)>,
        out_recv: Receiver<UpdateResult>,
    ) -> Self {
        Self {
            thread: MaybeUninit::zeroed(),
            system_sender: sys_send,
            output_reciever: out_recv,
        }
    }

    fn set_thread(
        &mut self,
        state: ThreadState,
        thread_builder: std::thread::Builder,
    ) -> io::Result<()> {
        let thread = thread_builder.spawn(move || loop {
            let (system, query) = state.system.recv().expect("Failed to read from channel");
            let out = system.update(query);
            state
                .output
                .send(out)
                .expect("Failed to send output from system");
        })?;

        self.thread.write(thread);
        io::Result::Ok(())
    }
}

pub(crate) struct ThreadPool {
    threads: Vec<SystemThread>,
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
            let (state, sender, recv) = ThreadState::new();
            let t = SystemThread::new_unset(sender, recv);
            threads.push(t);
            threads[thread].set_thread(state, thread_builder)?;
        }
        io::Result::Ok(Self { threads })
    }

    async fn run_system(
        system: Box<dyn SystemInterface>,
        query: Query<u8>,
        thread: &SystemThread,
    ) -> UpdateResult {
        thread
            .system_sender
            .send((system, query))
            .expect("Failed to send system to thread");
        thread
            .output_reciever
            .recv()
            .expect("Failed to receive output from thread")
    }

    fn prepare_futures(&self, schedule: &mut Schedule) -> Vec<Box<dyn Future<Output = ()>>> {
        let active_threads = Vec::new();
        for i in 0..self.threads.len() {
            if let NextSystem::System(sys) = schedule.next() {
                //let fut = Self::run_system(sys, thread);
            } else {
                break;
            }
        }
        active_threads
    }

    pub(crate) fn follow_schedule(&self, schedule: &mut Schedule, alloc: &mut Allocator) {
        let active_threads: Vec<Box<dyn Future<Output = ()>>> = Vec::new();
        schedule.prep_systems();

        // load initial systems
        loop {
            'thread_check: for thread in &self.threads {
                // see of thread has finished
                /*
                if !thread.state.lock().unwrap().thread_finished {
                    continue 'thread_check;
                }
                */

                // update schedule to reflect finished system

                // ask schedule what to do next
                match schedule.next() {
                    // a new system is ready, load it into this slot
                    NextSystem::System(next) => {
                        let interop_id = next.alloc_id();
                    }
                    // no available systems, continue iterating through
                    // others until deps are free'd
                    NextSystem::Wait => continue 'thread_check,
                    // systems finished. Can move onto rendering,
                    // then restart cycle
                    NextSystem::Finished => todo!(),
                }
            }
        }
    }
}
