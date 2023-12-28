use anyhow::Result;
use bblearn_api::Client;
use std::{
    env,
    sync::mpsc::{channel, Receiver, Sender},
    thread::JoinHandle,
};

use crate::{
    config::AuthCache,
    event::{Event as CrateEvent, EventLoop},
};

use super::{Event, Message};

/// Performs requests it receives from the main thread, and sends the results back.
pub struct StoreWorker {
    client: Client,
    msg_recv: Receiver<Message>,
    event_send: Sender<CrateEvent>,
}

impl StoreWorker {
    pub fn spawn_with(events: &EventLoop) -> Result<(JoinHandle<()>, Sender<Message>)> {
        let client = match AuthCache::load() {
            Ok(c) => Client::with_auth_state(c.creds, c.auth_state).unwrap(),
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

        let worker = StoreWorker {
            client,
            msg_recv: cmd_recv,
            event_send: events.sender(),
        };
        let handle = std::thread::spawn(move || worker.main());

        Ok((handle, cmd_send))
    }

    fn main(self) {
        while let Ok(msg) = self.msg_recv.recv() {
            if let Message::Quit = msg {
                break;
            }
            match self.process_msg(msg) {
                Ok(e) => self.event_send.send(CrateEvent::Store(e)),
                Err(e) => self.event_send.send(CrateEvent::Store(Event::Error(e))),
            }
            .unwrap();
        }

        AuthCache::from_client(&self.client).save().unwrap();
    }

    fn process_msg(&self, msg: Message) -> Result<Event> {
        match msg {
            Message::LoadMe => Ok(Event::Me(self.client.me()?)),
            Message::LoadMyCourses(me_id) => Ok(Event::MyCourses(
                self.client
                    .user_memberships(&me_id)?
                    .into_iter()
                    .map(|m| m.course)
                    .collect(),
            )),
            Message::LoadCourseContent(course_id) => {
                let content = self.client.course_content(&course_id)?;
                Ok(Event::CourseContent(course_id, content))
            }
            Message::LoadContentChildren(content_idx, course_id, content_id) => {
                let contents = self.client.content_children(&course_id, &content_id)?;
                Ok(Event::ContentChildren(content_idx, contents))
            }
            Message::Quit => unreachable!(),
        }
    }
}