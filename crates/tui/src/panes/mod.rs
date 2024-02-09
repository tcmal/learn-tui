use anyhow::Result;
use ratatui::{prelude::Rect, Frame};

use crate::{
    event::Event,
    main_screen::{Action, AppState},
};

mod navigation;
mod viewer;

pub use navigation::Navigation;
pub use viewer::{Document, Viewer};

pub trait Pane {
    fn draw(&mut self, state: &AppState, frame: &mut Frame, area: Rect);
    fn handle_event(&mut self, state: &AppState, key: Event) -> Result<Action>;
}
