//! This is the main crate, responsible for the TUI application stuff.
//!
//! # Architecture
//! We use [`ratatui`] with something like a multi-threaded [elm model](https://ratatui.rs/concepts/application-patterns/the-elm-architecture/).
//! First, our application is divided into [`Screen`]s - currently only the [`LoginPrompt`] and the [`MainScreen`].
//!
//! [`self::event::EventBus`] provides a multi-producer single-consumer event bus, and holds onto thread handles, etc.
//! Our [`main_loop`] then consists of:
//!
//!   * Drawing the current screen
//!   * Waiting for an event, and passing it up to the screen
//!   * So long as the screen doesn't say to change or quit, loop
//!
//! Currently at most 3 places can produce events:
//!
//!   * [`EventBus::spawn_terminal_listener`], which listens for key events, etc.
//!   * [`store::Worker`], which performs API requests and sends the results back
//!   * [`store::Downloader`], which downloads and saves files and sends progress updates
//!
//! The latter 2 receive commands from their own channels, and are driven by methods in [`store::Store`].
use anyhow::Result;
use event::{Event, EventBus};
use log::debug;
use main_screen::MainScreen;
use ratatui::prelude::*;
use simplelog::{LevelFilter, WriteLogger};
use std::{env, fs::File, io, rc::Rc};

use crate::{
    auth_cache::{AuthCache, LoginDetails},
    login_prompt::LoginPrompt,
};

pub mod auth_cache;
pub mod event;
pub mod login_prompt;
pub mod main_screen;
pub mod store;
pub mod tui;

pub fn main() -> Result<()> {
    init_logging();

    // Initialise terminal
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
    let bus = Rc::new(EventBus::new());
    bus.spawn_terminal_listener();

    // Login screen if needed, or just the app
    let app: Box<dyn Screen> = match AuthCache::load() {
        Ok(a) => Box::new(MainScreen::new(
            bus.clone(),
            LoginDetails {
                creds: a.creds,
                remember: true,
            },
        )),
        Err(_) => Box::new(LoginPrompt::new(bus.clone())),
    };

    // Start everything
    main_loop(app, bus, terminal)
}

/// A single screen of the app.
/// This will be the only thing the main loop asks to draw / handle events, so it will usually dispatch out to other places.
pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_event(&mut self, event: Event) -> Result<ExitState>;
}

/// Whether the current [`Screen`] should exit or change
pub enum ExitState {
    Running,
    Quit,
    ChangeScreen(Box<dyn Screen>),
}

/// Run the given screen using the given terminal.
pub fn main_loop<B: Backend>(
    mut app: Box<dyn Screen>,
    bus: Rc<EventBus>,
    terminal: &mut Terminal<B>,
) -> Result<()> {
    loop {
        let mut exit_state = ExitState::Running;
        while matches!(exit_state, ExitState::Running) {
            tui::draw(terminal, app.as_mut())?;

            let next = bus.next()?;
            debug!("received event {:?}", next);

            exit_state = app.handle_event(next)?;
        }

        match exit_state {
            ExitState::Quit => break,
            ExitState::ChangeScreen(s) => app = s,
            ExitState::Running => unreachable!(),
        }
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
