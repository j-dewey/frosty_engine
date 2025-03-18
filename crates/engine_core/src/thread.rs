use std::future::Future;
use std::io;
use std::mem::MaybeUninit;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::JoinHandle;

use crate::query::Query;
use crate::schedule::{NextSystem, Schedule};
use crate::system::{SystemInterface, UpdateResult};
use crate::Spawner;

// Threading Model
// Picturing a master thread moving functions and data into worker threads
// creates a pretty solid mental image of how threading works in this engine.
//
// Suppose you have twos systems A and B. There is no guarantee the order in
// which these systems will be run. In one frame A may run before B, in the next
// B could run before A, and after that they may even run at the same time.
//
// The only exception to this is if one is declared as depending on another. In
// that case, the depdendent system will always occur after the other. If B
// declares A as a depedenancy, then B will only ever start after A has finished.
// This should be used any time a System uses the same components as a fixed
// interval method (ex: any interop with physics)
//
// A and B can "safely" interop with eachother due to semaphores being built
// into the data passed into each system. This allows for an arbitrary amount
// of read accesses and only one write access. Safely in quotes here means
// that data will always be in some valid state, to make sure data is updated
// deterministically you should use commutative functions
//
// Not all data is stored behind semaphores. The examples useres are most likely
// to run into are Querys and InputHandler. While for the most part these objects
// can be used as readonly, there are legitimate reasons to mutate them. To do this
// safely, it must be done in a single-threaded context. The master thread is able
// to create this context, however it is unreachable by systems. Lockless queues
// are used double buffers in these instances to allow for message passing. When
// the master thread is in a single threaded context it can then update all the objects.

// The data needed to run a system
type SystemData = (Arc<dyn SystemInterface>, Query<u8>);

pub(crate) enum ThreadMode {
    Query,
    Update,
}

struct ThreadState {
    system: Receiver<SystemData>,
    output: Sender<UpdateResult>,
}

impl ThreadState {
    fn new() -> (Self, Sender<SystemData>, Receiver<UpdateResult>) {
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
    system_sender: Sender<SystemData>,
    output_reciever: Receiver<UpdateResult>,
}

impl SystemThread {
    // create a new thread without any system set
    fn new_unset(sys_send: Sender<SystemData>, out_recv: Receiver<UpdateResult>) -> Self {
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
            let (system, query) = match state.system.recv() {
                Ok((sys, query)) => (sys, query),
                Err(_) => continue, // Nothing loaded, just spin
            };
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
            let thread_builder = std::thread::Builder::new().name(format!("worker_{:?}", thread));
            let (state, sender, recv) = ThreadState::new();
            let t = SystemThread::new_unset(sender, recv);
            threads.push(t);
            threads[thread].set_thread(state, thread_builder)?;
        }
        io::Result::Ok(Self { threads })
    }

    async fn run_system(
        system: Arc<dyn SystemInterface>,
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

    fn prepare_futures<'a>(
        &'a self,
        alloc: &mut Spawner,
        schedule: &mut Schedule,
    ) -> Vec<Option<Box<impl Future<Output = UpdateResult> + 'a>>> {
        let mut active_threads = Vec::new();
        for i in 0..self.threads.len() {
            if let NextSystem::System(sys) = schedule.next() {
                let fut = Self::run_system(
                    sys.get_system(),
                    alloc.get_query_by_id(&sys.alloc_id()).unwrap(),
                    &self.threads[i],
                );
                active_threads.push(Some(Box::new(fut)));
            } else {
                active_threads.push(None);
            }
        }

        active_threads
    }

    pub(crate) fn follow_schedule(&self, schedule: &mut Schedule, alloc: &mut Spawner) {
        // load initial systems
        schedule.prep_systems();
        let mut all_finished = false;
        let mut loops = 0;
        let mut futures = self.prepare_futures(alloc, schedule);

        while !all_finished {
            println!("loop {:?}", loops);
            'thread_check: for (id, thread) in self.threads.iter().enumerate() {
                // see if thread is finished
                let thread_view = &mut futures[id];
                /*if thread_view.is_some() {
                    thread_view.
                }*/

                // update schedule to reflect finished system

                // ask schedule what to do next
                match schedule.next() {
                    // a new system is ready, load it into this slot
                    NextSystem::System(next) => {
                        let interop_id = next.alloc_id();
                        //futures[id] = Some( Self::run_system(next., query, thread) );
                    }
                    // no available systems, continue iterating through
                    // others until deps are free'd
                    NextSystem::Wait => continue 'thread_check,
                    // systems finished. Can move onto rendering,
                    // then restart cycle
                    NextSystem::Finished => all_finished = true,
                }
            }
            loops += 1;
        }
    }
}
