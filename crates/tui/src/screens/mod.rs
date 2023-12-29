use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, Frame};

use crate::{app::App, store::Store};

mod navigation;
mod viewer;

pub use navigation::NavigationPage;
pub use viewer::{Document, ViewerPage};

pub enum Action {
    Exit,
    None,
    Show(Document),
    FocusNavigation,
}

trait Page {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect);
    fn handle_key(&mut self, store: &Store, key: KeyEvent) -> Result<Action>;
}

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        let layout = Layout::new(
            Direction::Horizontal,
            [Constraint::Percentage(50), Constraint::Percentage(50)],
        )
        .split(frame.size());

        self.navigation.draw(&self.store, frame, layout[0]);
        self.viewer.draw(&self.store, frame, layout[1]);
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit application on `Ctrl-C`
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.quit();
                }
                Ok(())
            }
            // Other handlers you could add here.
            _ => {
                let action = match self.viewer_focused {
                    true => self.viewer.handle_key(&self.store, key),
                    false => self.navigation.handle_key(&self.store, key),
                }?;

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
        }
    }
}
