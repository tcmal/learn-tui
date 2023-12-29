use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::debug;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders},
    Frame,
};

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
        let size = frame.size();

        // area for content
        let content_rect = Rect {
            x: size.x + 1,
            y: size.y + 1,
            width: size.width - 2,
            height: size.height - 2,
        };

        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(30),
                Constraint::Length(1),
                Constraint::Percentage(70),
            ],
        )
        .split(content_rect);

        // area for focus rectangle to be drawn around
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

        debug!("content: {:?}, focus: {:?}", content_rect, focus_rect);

        self.navigation.draw(&self.store, frame, layout[0]);
        self.viewer.draw(&self.store, frame, layout[2]);
        frame.render_widget(Block::default().borders(Borders::ALL), focus_rect);
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
