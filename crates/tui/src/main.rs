use anyhow::Result;
use app::App;
use event::{Event, EventLoop};
use log::debug;
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
        File::create(".learn-tui.log").unwrap(),
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
        let next = tui.events.next()?;
        debug!("received event {:?}", next);
        match next {
            Event::Key(key_event) => app.handle_key(key_event)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
            Event::Store(e) => app.store.event(e),
        }
    }

    app.clean_shutdown();

    Ok(())
}
