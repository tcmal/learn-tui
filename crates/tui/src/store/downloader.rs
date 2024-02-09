use anyhow::Result;
use bblearn_api::Client;
use log::debug;
use std::sync::mpsc::{channel, Receiver, Sender};

use super::{DownloaderRequest, Event};
use crate::event::{Event as CrateEvent, EventBus};

/// Performs requests it receives from the main thread, and sends the results back.
pub struct Downloader {
    client: Client,
    msg_recv: Receiver<DownloaderRequest>,
    event_send: Sender<CrateEvent>,
}

impl Downloader {
    /// Spawn the store worker on the given event bus, returning a channel to send commands down.
    pub fn spawn_on(bus: &EventBus, client: Client) -> Result<Sender<DownloaderRequest>> {
        let (cmd_send, cmd_recv) = channel();

        bus.spawn("downloader", move |_, event_send| {
            // we don't need running because the receiver will raise an error and we'll exit
            Downloader {
                client,
                msg_recv: cmd_recv,
                event_send,
            }
            .main()
        });

        Ok(cmd_send)
    }

    fn main(self) {
        while let Ok(msg) = self.msg_recv.recv() {
            debug!("received message: {:?}", msg);
            if let Err(e) = match self.process_msg(msg) {
                Ok(e) => self.event_send.send(CrateEvent::Store(e)),
                Err(e) => self.event_send.send(CrateEvent::Store(Event::Error(e))),
            } {
                debug!("error sending event: {:?}", e);
                break;
            }
        }

        debug!("shutting down");
    }

    fn process_msg(&self, msg: DownloaderRequest) -> Result<Event, bblearn_api::Error> {
        match msg {}
    }
}
