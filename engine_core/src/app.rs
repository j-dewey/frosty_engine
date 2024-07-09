use crate::{schedule::Schedule, thread::SystemThread};

pub struct App {
    schedule: Schedule,
    thread_pool: Vec<SystemThread<'static>>,
}

impl App {
    pub fn new() -> Self {
        todo!()
    }
}
