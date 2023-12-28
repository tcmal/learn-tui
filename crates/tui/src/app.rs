use std::thread::JoinHandle;

use anyhow::Result;

use crate::{
    event::EventLoop,
    screens::ActivePage,
    store::{Store, StoreWorker},
};

/// Holds all application state
pub struct App {
    pub running: bool,
    pub curr_page: ActivePage,
    pub store: Store,
    store_worker_handle: JoinHandle<()>,
}
impl App {
    pub fn new(events: &EventLoop) -> Result<Self> {
        let (worker_handle, worker_queue) = StoreWorker::spawn_with(events)?;

        Ok(Self {
            running: true,
            curr_page: ActivePage::new()?,
            store: Store::new(worker_queue),
            store_worker_handle: worker_handle,
        })
    }

    pub fn quit(&mut self) {
        self.running = false;
    }

    pub fn clean_shutdown(self) {
        self.store.request_quit();
        self.store_worker_handle.join().unwrap();
    }
}
