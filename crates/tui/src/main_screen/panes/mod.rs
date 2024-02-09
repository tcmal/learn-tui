use ratatui::{prelude::Rect, Frame};

use crate::{event::Event, main_screen::Action, store::Store};

mod navigation;
mod viewer;

pub use navigation::Navigation;
pub use viewer::{Document, Viewer};

/// An individual pane in the main screen
/// This is similar to the [`crate::Screen`] trait, but we draw multiple panes at the same time.
pub trait Pane {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, store: &mut Store, event: Event) -> Action;
}
