use anyhow::Result;
use app::App;
use event::{Event, EventHandler};
use handler::handle_key_events;
use ratatui::prelude::*;
use std::io;
use tui::Tui;

mod app;
mod config;
mod event;
mod handler;
mod tui;

fn main() -> Result<()> {
    let mut app = App::new()?;

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(250);
    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Tick => (),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    tui.exit()?;
    Ok(())
}
