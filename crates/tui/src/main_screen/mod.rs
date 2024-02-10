use std::rc::Rc;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use edlearn_client::Client;
use log::{debug, error};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::{
    auth_cache::{AuthCache, LoginDetails},
    event::{Event, EventBus},
    login_prompt::LoginPrompt,
    store::Store,
    ExitState, Screen,
};

pub mod panes;
use panes::{Document, Navigation};

use self::panes::{Pane, Viewer};

/// An action that a [`Pane`] can request to be taken
pub enum Action {
    /// Do nothing
    None,

    /// Quit the application
    Exit,

    /// Tell the viewer to show something, and focus the viewer
    Show(Document),

    /// Focus the navigation pane
    FocusNavigation,

    /// Go back to the login screen
    Reauthenticate,

    /// Display the given string at the bottom of the screen
    Flash(Text<'static>),
}

/// The main screen of the application
/// The bulk of the UI logic is handled by the [`self::panes`], this just contains shared state.
pub struct MainScreen {
    /// Handle to the client we're using, so we can save auth state when we exit
    client: Client,

    /// Underlying data store,
    store: Store,

    /// UI Components & State
    navigation: Navigation,
    viewer: Viewer,
    viewer_focused: bool,
    save_auth_state: bool,

    flash: Text<'static>,

    events: Rc<EventBus>,
}

impl MainScreen {
    /// Create a new app using the given event bus and login details
    pub fn new(events: Rc<EventBus>, login_details: LoginDetails) -> Self {
        let client = match AuthCache::load() {
            Ok(c) => c.into_client().unwrap(),
            Err(e) => {
                debug!("error loading config: {:?}", e);

                Client::new(login_details.creds)
            }
        };

        Self {
            store: Store::new(&events, client.clone_sharing_state()),
            events,
            client,
            navigation: Navigation::default(),
            viewer: Viewer::default(),
            viewer_focused: false,
            save_auth_state: login_details.remember,
            flash: Text::raw(""),
        }
    }

    /// Quit the application, saving the auth state
    pub fn quit(&mut self) -> Result<ExitState> {
        if self.save_auth_state {
            debug!("saving auth state");
            if let Err(e) = AuthCache::from_client(&self.client).save() {
                error!("error saving auth state: {}", e);
            }
        }

        Ok(ExitState::Quit)
    }
}

impl Screen for MainScreen {
    fn draw(&mut self, frame: &mut Frame) {
        let size = frame.size();

        // Add margin for borders
        let content_rect = Rect {
            x: size.x + 1,
            y: size.y + 1,
            width: size.width - 2,
            height: size.height - 2,
        };

        // 30/70 split the two panes
        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(30),
                Constraint::Length(1),
                Constraint::Percentage(70),
            ],
        )
        .split(content_rect);

        self.navigation.draw(&self.store, frame, layout[0]);
        self.viewer.draw(&self.store, frame, layout[2]);

        // Draw a focus rectangle around one of them.
        let focus_rect = if !self.viewer_focused {
            Rect {
                x: size.x,
                y: size.y,
                width: layout[2].x - size.x,
                height: size.height,
            }
        } else {
            Rect {
                x: layout[1].x,
                y: size.y,
                width: size.width - layout[1].x,
                height: size.height,
            }
        };

        frame.render_widget(Block::default().borders(Borders::ALL), focus_rect);

        let bottom_bar = Paragraph::new(self.flash.clone());
        frame.render_widget(
            bottom_bar,
            Rect {
                x: layout[2].x + 1,
                y: size.height.saturating_sub(1),
                width: layout[2].width.saturating_sub(1),
                height: 1,
            },
        )
    }

    /// Handle the given event
    fn handle_event(&mut self, event: Event) -> Result<ExitState> {
        // C-C always exits
        if matches!(
            event,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c') | KeyCode::Char('C'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })
        ) {
            return self.quit();
        }

        // Dispatch to pane or store
        let action = match event {
            Event::Store(s) => self.store.event(s),
            x => match self.viewer_focused {
                true => self.viewer.handle_event(&mut self.store, x),
                false => self.navigation.handle_event(&mut self.store, x),
            },
        };

        self.flash = Text::raw("");

        // Perform action if needed
        match action {
            Action::None => (),
            Action::Exit => {
                return self.quit();
            }
            Action::Show(doc) => {
                self.viewer.show(doc);
                self.viewer_focused = true;
            }
            Action::FocusNavigation => self.viewer_focused = false,
            Action::Reauthenticate => {
                return Ok(ExitState::ChangeScreen(Box::new(
                    LoginPrompt::new_with_msg(
                        self.events.clone(),
                        "Authentication failed, please double check your username & password.",
                    ),
                )));
            }
            Action::Flash(s) => {
                self.flash = s;
            }
        };

        Ok(ExitState::Running)
    }
}
