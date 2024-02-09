use anyhow::Result;
use ratatui::{prelude::Rect, Frame};

use crate::{event::Event, main_screen::Action, store::Store};

mod navigation;
mod viewer;

pub use navigation::Navigation;
pub use viewer::{Document, Viewer};

pub trait Pane {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, store: &mut Store, key: Event) -> Result<Action>;
}
