use anyhow::Result;
use bblearn_api::Client;
use std::sync::mpsc::Sender;

mod worker;
use worker::Worker;

use crate::{event::EventBus, main_screen::Action};

/// File download state
pub struct Downloader {
    worker_channel: Sender<Request>,
}

/// Requests sent to the worker thread
#[derive(Debug)]
enum Request {}

/// Messages received by the app from the worker thread
#[derive(Debug)]
pub enum Event {
    Error(bblearn_api::Error),
}

impl Downloader {
    pub fn new(bus: &EventBus, client: Client) -> Result<Self> {
        let worker_channel = Worker::spawn_on(bus, client)?;

        Ok(Self { worker_channel })
    }

    pub fn event(&mut self, e: Event) -> Action {
        match e {
            Event::Error(bblearn_api::Error::AuthError(_)) => return Action::Reauthenticate,
            Event::Error(e) => panic!("{}", e), // TODO
        };

        Action::None
    }
}
