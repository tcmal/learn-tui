use crossterm::event::KeyCode;
use ratatui::{prelude::Rect, Frame};

use crate::{
    event::Event,
    store::{ContentIdx, Store},
};

use super::{Action, Pane};

mod content;
mod downloads;
mod welcome;

use content::ContentViewer;
use downloads::DownloadsViewer;
use welcome::WelcomeViewer;

/// Something we want to show in the viewer
#[derive(Default)]
pub enum Document {
    /// The welcome message
    #[default]
    Welcome,

    /// The list of downloads
    Downloads,

    /// A content item
    Content(ContentIdx),
}

/// Shows [`Document`]s to the user.
/// Most of the view logic is in submodules, to keep things clean.
pub enum Viewer {
    Welcome(WelcomeViewer),
    Downloads(DownloadsViewer),
    Content(ContentViewer),
}

impl Default for Viewer {
    fn default() -> Self {
        Self::Welcome(Default::default())
    }
}

impl Viewer {
    /// Set the content that we will show from next draw.
    pub fn show(&mut self, d: Document) {
        match d {
            Document::Welcome => *self = Self::Welcome(Default::default()),
            Document::Downloads => *self = Self::Downloads(Default::default()),
            Document::Content(idx) => *self = Self::Content(ContentViewer::new(idx)),
        };
    }
}

impl Pane for Viewer {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect) {
        match self {
            Viewer::Welcome(viewer) => viewer.draw(store, frame, area),
            Viewer::Downloads(viewer) => viewer.draw(store, frame, area),
            Viewer::Content(viewer) => viewer.draw(store, frame, area),
        }
    }

    fn handle_event(&mut self, store: &mut Store, event: Event) -> Action {
        let Event::Key(key) = event else {
            return Action::None;
        };

        if let KeyCode::Char('q') | KeyCode::Esc = key.code {
            return Action::FocusNavigation;
        };

        match self {
            Viewer::Welcome(viewer) => viewer.handle_event(store, event),
            Viewer::Downloads(viewer) => viewer.handle_event(store, event),
            Viewer::Content(viewer) => viewer.handle_event(store, event),
        }
    }
}
