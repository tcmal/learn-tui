use anyhow::Result;
use bblearn_api::Client;
use log::debug;
use std::sync::mpsc::{channel, Receiver, Sender};

use super::{Event, Request};
use crate::event::{Event as CrateEvent, EventBus};

/// Performs requests it receives from the main thread, and sends the results back.
pub struct Worker {
    client: Client,
    msg_recv: Receiver<Request>,
    event_send: Sender<CrateEvent>,
}

impl Worker {
    /// Spawn the store worker on the given event bus, returning a channel to send commands down.
    pub fn spawn_on(bus: &EventBus, client: Client) -> Result<Sender<Request>> {
        let (cmd_send, cmd_recv) = channel();

        bus.spawn("store_worker", move |_, event_send| {
            // we don't need running because the receiver will raise an error and we'll exit
            Worker {
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

    fn process_msg(&self, msg: Request) -> Result<Event, bblearn_api::Error> {
        match msg {
            Request::Me => {
                let me = self.client.me()?;
                let courses = self
                    .client
                    .user_memberships(&me.id)?
                    .into_iter()
                    .map(|m| m.course)
                    .collect::<Vec<_>>();

                let terms = self.client.terms()?;

                Ok(Event::Me(me, courses, terms))
            }
            Request::CourseContent {
                course_idx,
                course_id,
            } => {
                let content = self.client.content_children(&course_id, "ROOT")?;
                Ok(Event::CourseContent {
                    course_idx,
                    content,
                })
            }
            Request::ContentChildren {
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
            Request::PageText {
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
