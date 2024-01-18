use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use log::debug;
use std::cell::RefCell;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::store;

/// An event our app may receive
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

/// The event bus aggregates events from multiple threads, and joins them all back when required.
/// FIXME: We don't actually use our join handles, we just let the threads get cleaned up since we'll exit right after.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventBus {
    sender: mpsc::Sender<Event>,
    receiver: mpsc::Receiver<Event>,
    running: Arc<AtomicBool>,
    handles: RefCell<Vec<thread::JoinHandle<()>>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            sender,
            receiver,
            running: Arc::new(AtomicBool::new(true)),
            handles: Default::default(),
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub fn next(&self) -> Result<Event> {
        Ok(self.receiver.recv()?)
    }

    /// Spawn a new thread that can publish to this event bus
    pub fn spawn<F>(&self, name: impl ToString, f: F)
    where
        F: 'static + Send + FnOnce(Arc<AtomicBool>, Sender<Event>),
    {
        let sender = self.sender.clone();
        let running = self.running.clone();
        self.handles.borrow_mut().push(
            thread::Builder::new()
                .name(name.to_string())
                .spawn(move || f(running, sender))
                .unwrap(),
        );
    }

    /// Spawn a thread to publish terminal events to this bus
    pub fn spawn_terminal_listener(&self) {
        self.spawn("terminal_events", Self::terminal_events)
    }

    /// Polls for terminal events and sends them to the given sender.
    fn terminal_events(running: Arc<AtomicBool>, sender: Sender<Event>) {
        loop {
            if event::poll(Duration::from_millis(250)).expect("unable to poll for events") {
                match event::read().expect("unable to read event") {
                    CrosstermEvent::Key(e) => sender.send(Event::Key(e)),
                    CrosstermEvent::Mouse(e) => sender.send(Event::Mouse(e)),
                    CrosstermEvent::Resize(w, h) => sender.send(Event::Resize(w, h)),
                    _ => Ok(()),
                }
                .expect("failed to send terminal event");
            }
            if !running.load(Ordering::Relaxed) {
                break;
            }
        }
    }
}

impl Drop for EventBus {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        self.handles.borrow_mut().drain(..).for_each(|h| {
            debug!("joining thread {:?}", h.thread().name());
            h.join().unwrap()
        });
    }
}
