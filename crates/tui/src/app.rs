use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::debug;
use ratatui::{
    prelude::{Constraint, Direction, Layout, Rect},
    widgets::{Block, Borders},
    Frame,
};

use crate::{
    event::{Event, EventBus},
    panes::{Document, Navigation, Pane, Viewer},
    store::Store,
};

pub enum Action {
    Exit,
    None,
    Show(Document),
    FocusNavigation,
}

/// Holds application-related state
pub struct App {
    pub running: bool,
    store: Store,
    navigation: Navigation,
    viewer: Viewer,
    viewer_focused: bool,
}

impl App {
    /// Create a new app using the given event bus
    pub fn new(events: &mut EventBus) -> Result<Self> {
        Ok(Self {
            store: Store::new(events)?,
            navigation: Navigation::default(),
            viewer: Viewer::default(),
            running: true,
            viewer_focused: false,
        })
    }

    /// Draw to the given frame
    pub fn draw(&mut self, frame: &mut Frame) {
        debug!("drawing");
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

        debug!("drawing navigation");
        self.navigation.draw(&self.store, frame, layout[0]);
        debug!("drawing viewer");
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
    pub fn handle_event(&mut self, event: Event) -> Result<()> {
        // C-C always exits
        if matches!(
            event,
            Event::Key(KeyEvent {
                code: KeyCode::Char('c') | KeyCode::Char('C'),
                modifiers: KeyModifiers::CONTROL,
                ..
            })
        ) {
            self.quit();
            return Ok(());
        }

        // Dispatch store events to the store
        if let Event::Store(s) = event {
            self.store.event(s);
            return Ok(());
        }

        // and everything else to whichever pane is focused
        let action = match self.viewer_focused {
            true => self.viewer.handle_event(&self.store, event),
            false => self.navigation.handle_event(&self.store, event),
        }?;

        // Perform action if needed
        match action {
            Action::None => (),
            Action::Exit => self.quit(),
            Action::Show(doc) => {
                self.viewer.show(doc);
                self.viewer_focused = true;
            }
            Action::FocusNavigation => self.viewer_focused = false,
        };

        Ok(())
    }

    /// Quit the application
    pub fn quit(&mut self) {
        self.running = false;
    }
}
