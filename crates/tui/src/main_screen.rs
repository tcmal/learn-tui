use std::rc::Rc;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    auth_cache::LoginDetails,
    event::{Event, EventBus},
    login_prompt::LoginPrompt,
    panes::{Document, Navigation, Pane, Viewer},
    store::Store,
    ExitState, Screen,
};

pub enum Action {
    Exit,
    None,
    Show(Document),
    FocusNavigation,
    Reauthenticate,
}

/// Holds application-related state
pub struct MainScreen {
    pub running: bool,
    store: Store,
    navigation: Navigation,
    viewer: Viewer,
    viewer_focused: bool,
    events: Rc<EventBus>,
}

impl MainScreen {
    /// Create a new app using the given event bus
    pub fn new(events: Rc<EventBus>, login_details: LoginDetails) -> Result<Self> {
        Ok(Self {
            store: Store::new(&events, login_details)?,
            events,
            navigation: Navigation::default(),
            viewer: Viewer::default(),
            running: true,
            viewer_focused: false,
        })
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}

impl Screen for MainScreen {
    /// Draw to the given frame
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
            return Ok(ExitState::Quit);
        }

        let action = if let Event::Store(s) = event {
            // Dispatch store events to the store
            self.store.event(s).unwrap_or(Action::None)
        } else {
            // and everything else to whichever pane is focused
            match self.viewer_focused {
                true => self.viewer.handle_event(&self.store, event),
                false => self.navigation.handle_event(&self.store, event),
            }?
        };

        // Perform action if needed
        match action {
            Action::None => (),
            Action::Exit => self.quit(),
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
        };

        Ok(ExitState::Running)
    }
}
