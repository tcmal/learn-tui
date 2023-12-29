use anyhow::Result;
use bblearn_api::content::ContentPayload;
use crossterm::event::KeyCode;
use ratatui::{prelude::Rect, widgets::Paragraph, Frame};

use crate::{
    event::Event,
    store::{ContentIdx, Store},
};

use super::{Action, Pane};

#[derive(Default)]
pub enum Document {
    #[default]
    Blank,
    Content(ContentIdx),
}

#[derive(Default)]
pub struct Viewer {
    show: Document,
    cached_render: Option<Paragraph<'static>>,
}
impl Viewer {
    pub fn show(&mut self, d: Document) {
        self.show = d;
        self.cached_render = None;
    }
}

impl Pane for Viewer {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect) {
        let rendered = self
            .cached_render
            .clone()
            .unwrap_or_else(|| match self.show {
                Document::Blank => {
                    self.cached_render = Some(Paragraph::new(""));
                    self.cached_render.clone().unwrap()
                }
                Document::Content(idx) => self.render_content(store, idx),
            });

        frame.render_widget(rendered, area)
    }

    fn handle_event(&mut self, _: &Store, event: Event) -> Result<Action> {
        let Event::Key(key) = event else {
            return Ok(Action::None);
        };

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Action::FocusNavigation),
            _ => (),
            // TODO
        };

        Ok(Action::None)
    }
}

impl Viewer {
    fn render_content(&mut self, store: &Store, content_idx: ContentIdx) -> Paragraph<'static> {
        let content = store.content(content_idx);
        match &content.payload {
            ContentPayload::Page => match store.page_text(content_idx) {
                Some(text) => {
                    self.cached_render = Some(bbml::render(text));
                    self.cached_render.clone().unwrap()
                }
                None => {
                    store.request_page_text(content_idx);
                    Paragraph::new("Loading...")
                }
            },
            ContentPayload::Link(l) => Paragraph::new(format!("Link to {}", l)),
            ContentPayload::Folder => Paragraph::new("Folder"),
            ContentPayload::Other(o) => Paragraph::new(format!("Unrecognised content type: {}", o)),
        }
    }
}
