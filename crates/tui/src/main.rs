use anyhow::{anyhow, Result};
use event::{Event, EventBus};
use log::{debug, error};
use ratatui::prelude::*;
use simplelog::{LevelFilter, WriteLogger};
use std::{env, fs::File, io};
use viewer::App;

use crate::{
    auth_cache::{AuthCache, LoginDetails},
    login_prompt::LoginPrompt,
};

mod auth_cache;
mod event;
mod login_prompt;
mod panes;
mod store;
mod tui;
mod viewer;

fn main() -> Result<()> {
    init_logging();

    // Initialise terminal and event bus
    let mut terminal = Terminal::new(CrosstermBackend::new(io::stderr()))?;
    tui::init(&mut terminal)?;

    let res = run_in_terminal(&mut terminal);

    // Cleanup
    debug!("exiting");
    tui::exit(&mut terminal)?;

    if let Err(e) = res {
        println!("{}", e);
    }

    Ok(())
}

fn run_in_terminal<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut bus = EventBus::new();
    bus.spawn_terminal_listener();

    // Login screen if needed
    let login_details = match AuthCache::load() {
        Ok(a) => LoginDetails {
            creds: a.creds,
            remember: true,
        },
        Err(_) => prompt_auth(&mut bus, terminal)?,
    };

    // Initialise app and event bus
    let mut app = App::new(&mut bus, login_details)?;

    main_loop(&mut app, &mut bus, terminal)
}

fn prompt_auth<B: Backend>(bus: &mut EventBus, terminal: &mut Terminal<B>) -> Result<LoginDetails> {
    // Initialise app and event bus
    let mut app = LoginPrompt::default();

    if let Err(e) = main_loop(&mut app, bus, terminal) {
        error!("error in main loop: {}", e);
    }

    app.extract_details().ok_or_else(|| anyhow!("exited"))
}

pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_event(&mut self, event: Event) -> Result<()>;
    fn running(&self) -> bool;
}

fn main_loop<A: Screen, B: Backend>(
    app: &mut A,
    bus: &mut EventBus,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    while app.running() {
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
