use anyhow::Result;
use event::{Event, EventBus};
use log::debug;
use ratatui::prelude::*;
use simplelog::{LevelFilter, WriteLogger};
use std::{env, fs::File, io, rc::Rc};
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
        Ok(a) => Box::new(App::new(
            bus.clone(),
            LoginDetails {
                creds: a.creds,
                remember: true,
            },
        )?),
        Err(_) => Box::new(LoginPrompt::new(bus.clone())),
    };

    // Start everything
    main_loop(app, bus, terminal)
}

pub trait Screen {
    fn draw(&mut self, frame: &mut Frame);
    fn handle_event(&mut self, event: Event) -> Result<ExitState>;
}

pub enum ExitState {
    Running,
    Quit,
    ChangeScreen(Box<dyn Screen>),
}

fn main_loop<B: Backend>(
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
