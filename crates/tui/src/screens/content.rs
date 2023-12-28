use anyhow::Result;
use crossterm::event::KeyEvent;
use ratatui::{prelude::Rect, widgets::Paragraph, Frame};

use crate::store::Store;

use super::{Action, Page};

#[derive(Default)]
pub struct ContentPage {
    render: Option<Paragraph<'static>>,
}
impl ContentPage {
    pub fn set_displaying(&mut self, p: Paragraph<'static>) {
        self.render = Some(p);
    }
}

impl Page for ContentPage {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect) {
        match self.render.clone() {
            Some(p) => frame.render_widget(p, area),
            None => (), // TODO
        }
    }

    fn handle_key(&mut self, store: &Store, key: KeyEvent) -> Result<Action> {
        match key.code {
            _ => (),
            // TODO
        }
        Ok(Action::None)
    }
}
