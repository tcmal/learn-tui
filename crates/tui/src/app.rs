use std::thread::JoinHandle;

use anyhow::Result;

use crate::{
    event::EventLoop,
    screens::{ContentPage, NavigationPage},
    store::{Store, StoreWorker},
};

/// Holds all application state
pub struct App {
    pub running: bool,
    pub store: Store,
    store_worker_handle: JoinHandle<()>,

    pub navigation: NavigationPage,
    pub content: ContentPage,
    pub content_focused: bool,
}
impl App {
    pub fn new(events: &EventLoop) -> Result<Self> {
        let (worker_handle, worker_queue) = StoreWorker::spawn_with(events)?;

        Ok(Self {
            running: true,
            store: Store::new(worker_queue),
            store_worker_handle: worker_handle,
            navigation: NavigationPage::default(),
            content: ContentPage::default(),
            content_focused: false,
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
