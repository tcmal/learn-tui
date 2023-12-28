use anyhow::Result;
use app::App;
use event::{Event, EventLoop};
use ratatui::prelude::*;
use simplelog::{LevelFilter, WriteLogger};
use std::{fs::File, io};
use tui::Tui;

mod app;
mod config;
mod event;
mod screens;
mod store;
mod tui;
mod widgets;

fn main() -> Result<()> {
    WriteLogger::init(
        LevelFilter::Debug,
        simplelog::Config::default(),
        File::create("my_rust_binary.log").unwrap(),
    )
    .unwrap();

    let backend = CrosstermBackend::new(io::stderr());
    let terminal = Terminal::new(backend)?;

    let events = EventLoop::new();
    let mut app = App::new(&events)?;

    let mut tui = Tui::new(terminal, events);

    tui.init()?;

    while app.running {
        tui.draw(&mut app)?;
        match tui.events.next()? {
            Event::Key(key_event) => app.handle_key(key_event)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Store(e) => app.store.event(e),
        }
    }

    app.clean_shutdown();

    Ok(())
}
