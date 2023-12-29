use anyhow::Result;
use bblearn_api::Client;
use log::debug;
use std::{
    env,
    sync::mpsc::{channel, Receiver, Sender},
};

use super::{Event, LoadRequest};
use crate::{
    auth_cache::AuthCache,
    event::{Event as CrateEvent, EventBus},
};

/// Performs requests it receives from the main thread, and sends the results back.
pub struct StoreWorker {
    client: Client,
    msg_recv: Receiver<LoadRequest>,
    event_send: Sender<CrateEvent>,
}

impl StoreWorker {
    /// Spawn the store worker on the given event bus, returning a channel to send commands down.
    pub fn spawn_on(bus: &mut EventBus) -> Result<Sender<LoadRequest>> {
        let client = match AuthCache::load() {
            Ok(c) => c.into_client().unwrap(),
            Err(e) => {
                println!("error loading config: {:?}", e);

                let creds = (
                    env::var("LEARN_USERNAME").unwrap(),
                    env::var("LEARN_PASSWORD").unwrap().into(),
                );
                Client::new(creds)
            }
        };
        let (cmd_send, cmd_recv) = channel();

        bus.spawn("store_worker", move |_, event_send| {
            // we don't need running because the receiver will raise an error and we'll exit
            StoreWorker {
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
        debug!(
            "saving config: {:?}",
            AuthCache::from_client(&self.client).save()
        );
    }

    fn process_msg(&self, msg: LoadRequest) -> Result<Event> {
        match msg {
            LoadRequest::Me => {
                let me = self.client.me()?;
                let courses = self
                    .client
                    .user_memberships(&me.id)?
                    .into_iter()
                    .map(|m| m.course)
                    .collect();
                Ok(Event::Me(me, courses))
            }
            LoadRequest::CourseContent {
                course_idx,
                course_id,
            } => {
                let content = self.client.content_children(&course_id, "ROOT")?;
                Ok(Event::CourseContent {
                    course_idx,
                    content,
                })
            }
            LoadRequest::ContentChildren {
                content_idx,
                course_id,
                content_id,
            } => {
                let children = self.client.content_children(&course_id, &content_id)?;
                Ok(Event::ContentChildren {
                    content_idx,
                    children,
                })
            }
            LoadRequest::PageText {
                content_idx,
                course_id,
                content_id,
            } => {
                let text = self.client.page_text(&course_id, &content_id)?;
                Ok(Event::PageText { content_idx, text })
            }
        }
    }
}
