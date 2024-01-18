use anyhow::Result;
use bblearn_api::content::ContentPayload;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::{Margin, Rect},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
    Frame,
};

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
    y_offset: u16,
    jump_y_offset: u16,
    cached_render: Option<Paragraph<'static>>,
}
impl Viewer {
    pub fn show(&mut self, d: Document) {
        self.show = d;
        self.y_offset = 0;
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

        let line_count = rendered.line_count(area.width);
        self.jump_y_offset = area.height / 2;

        let max_y_offset = (line_count as u16).saturating_sub(area.height as u16);
        self.y_offset = self.y_offset.min(max_y_offset);

        let scrollbar = Scrollbar::default()
            .orientation(ScrollbarOrientation::VerticalRight)
            .begin_symbol(Some("↑"))
            .end_symbol(Some("↓"));
        let mut scrollbar_state =
            ScrollbarState::new(max_y_offset as usize).position(self.y_offset as usize);

        frame.render_widget(
            rendered.scroll((self.y_offset, 0)),
            area.inner(&Margin {
                vertical: 0,
                horizontal: 1,
            }),
        );
        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }

    fn handle_event(&mut self, store: &Store, event: Event) -> Result<Action> {
        let Event::Key(key) = event else {
            return Ok(Action::None);
        };

        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(Action::FocusNavigation),

            KeyCode::Char('g') => self.y_offset = 0,
            KeyCode::Char('G') => self.y_offset = u16::MAX,

            KeyCode::Char('j') => self.y_offset += 1,
            KeyCode::Char('k') => self.y_offset = self.y_offset.saturating_sub(1),

            KeyCode::Char('u') | KeyCode::Char('U')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.y_offset = self.y_offset.saturating_sub(self.jump_y_offset)
            }
            KeyCode::Char('d') | KeyCode::Char('D')
                if key.modifiers.contains(KeyModifiers::CONTROL) =>
            {
                self.y_offset += self.jump_y_offset
            }

            KeyCode::Char('b') => match self.show {
                Document::Blank => (),
                Document::Content(content_idx) => {
                    let content = store.content(content_idx);
                    let link = if let ContentPayload::Link(link) = &content.payload {
                        Some(link)
                    } else {
                        content.link.as_ref()
                    };

                    if let Some(link) = link {
                        open::that(link)?;
                    }
                }
            },
            _ => (),
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
