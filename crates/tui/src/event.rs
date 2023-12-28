use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::sync::mpsc::{self, Sender};
use std::thread;


use crate::store;

/// Events our TUI may receive
#[derive(Debug)]
pub enum Event {
    /// Key press.
    Key(KeyEvent),

    /// Mouse click/scroll.
    Mouse(MouseEvent),

    /// Terminal resize.
    Resize(u16, u16),

    /// Some data for the store, sent by the store worker.
    Store(store::Event),
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct EventLoop {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    term_handler: thread::JoinHandle<()>,
}

impl EventLoop {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        let handler = {
            let sender = sender.clone();
            thread::spawn(move || loop {
                // Poll for terminal events and send them
                match event::read().expect("unable to read event") {
                    CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                    CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                    CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                    CrosstermEvent::FocusGained => Ok(()),
                    CrosstermEvent::FocusLost => Ok(()),
                    CrosstermEvent::Paste(_) => unimplemented!(),
                }
                .expect("failed to send terminal event")
            })
        };
        Self {
            sender,
            receiver,
            term_handler: handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    /// Get a channel to send events down.
    pub fn sender(&self) -> Sender<Event> {
        self.sender.clone()
    }
}
