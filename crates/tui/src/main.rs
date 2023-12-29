use anyhow::Result;
use app::App;
use event::EventBus;
use log::{debug, error};
use ratatui::prelude::*;
use simplelog::{LevelFilter, WriteLogger};
use std::{env, fs::File, io};

mod app;
mod auth_cache;
mod event;
mod panes;
mod store;
mod tui;

fn main() -> Result<()> {
    init_logging();

    // Initialise app and event bus
    let mut bus = EventBus::new();
    let mut app = App::new(&mut bus)?;
    bus.spawn_terminal_listener();

    // Initialise terminal
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stderr()))?;
    tui::init(&mut terminal)?;

    if let Err(e) = main_loop(&mut app, &mut bus, &mut terminal) {
        error!("error in main loop: {}", e);
    }

    // Cleanup
    debug!("exiting");
    tui::exit(&mut terminal)?;
    Ok(())
}

fn main_loop<B: Backend>(
    app: &mut App,
    bus: &mut EventBus,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    while app.running {
        tui::draw(terminal, app)?;
        let next = bus.next()?;
        debug!("received event {:?}", next);
        app.handle_event(next)?;
    }

    Ok(())
}

fn init_logging() {
    // Log if environment variable set
    if env::var("LEARN_TUI_LOG").is_ok() {
        WriteLogger::init(
            LevelFilter::Debug,
            simplelog::Config::default(),
            File::create(".learn-tui.log").unwrap(),
        )
        .unwrap();
    }
}
