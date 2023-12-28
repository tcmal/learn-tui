use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::{prelude::*, widgets::Paragraph, Frame};

use crate::{app::App, store::Store};

mod content;
mod navigation;

pub use content::ContentPage;
pub use navigation::NavigationPage;

pub enum Action {
    Exit,
    None,
    ShowContent(Paragraph<'static>),
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
        self.content.draw(&self.store, frame, layout[1]);
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
                let action = match self.content_focused {
                    true => self.content.handle_key(&self.store, key),
                    false => self.navigation.handle_key(&self.store, key),
                }?;

                match action {
                    Action::None => (),
                    Action::Exit => self.quit(),
                    Action::ShowContent(p) => {
                        self.content.set_displaying(p);
                        self.content_focused = true;
                    }
                };

                Ok(())
            }
        }
    }
}
