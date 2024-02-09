use anyhow::Result;
use bblearn_api::content::ContentPayload;
use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    prelude::{Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
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
    Welcome,
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
                Document::Content(idx) => {
                    if let Some(p) = self.render_content(&store, idx) {
                        self.cached_render = Some(p.clone());
                        p
                    } else {
                        // dont cache
                        Paragraph::new("Loading...")
                    }
                }
                Document::Welcome => {
                    let p = Paragraph::new(Into::<Text>::into(vec![
                        vec!["Welcome to learn-tui!\n".blue().bold()].into(),
                        vec![
                            "Use ".into(),
                            "j/k or ↓/↑".blue(),
                            " to navigate up and down, then ".into(),
                            "Enter".blue(),
                            " to select an item.\n".into(),
                        ]
                        .into(),
                        vec![
                            "When an item is selected, you can scroll the viewer pane using "
                                .into(),
                            "j/k ↓/↑ g/G PgUp/PgDn".blue(),
                            " and go back to the navigation pane with ".into(),
                            "q".blue(),
                            ".".into(),
                        ]
                        .into(),
                        vec![
                            "At any point, use ".into(),
                            "b".blue(),
                            " to try to open the selected item in your browser.\n".into(),
                        ]
                        .into(),
                        vec!["Use ".into(), "Ctrl-C".blue(), " to quit.".into()].into(),
                    ]))
                    .wrap(Wrap { trim: false });
                    self.cached_render = Some(p.clone());
                    p
                }
            });

        let line_count = rendered.line_count(area.width);
        self.jump_y_offset = area.height / 2;

        let max_y_offset = (line_count as u16).saturating_sub(area.height);
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

            KeyCode::Char('b') => {
                if let Document::Content(content_idx) = self.show {
                    let content = store.content(content_idx);
                    open::that(content.browser_link())?;
                };
            }
            _ => (),
        };

        Ok(Action::None)
    }
}

impl Viewer {
    fn render_content(
        &mut self,
        store: &Store,
        content_idx: ContentIdx,
    ) -> Option<Paragraph<'static>> {
        let content = store.content(content_idx);
        match &content.payload {
            ContentPayload::Page => match store.page_text(content_idx) {
                Some(text) => {
                    self.cached_render = Some(bbml::render(text));
                    Some(self.cached_render.clone().unwrap())
                }
                None => {
                    store.request_page_text(content_idx);
                    None
                }
            },
            ContentPayload::Link(l) => Some(Paragraph::new(format!("Link to {}. Open with b", l))),
            ContentPayload::Folder => Some(Paragraph::new("Folder")),
            ContentPayload::File {
                file_name,
                mime_type,
                ..
            } => Some(Paragraph::new(vec![
                Line::styled(
                    file_name.clone(),
                    Style::new().fg(Color::Blue).add_modifier(Modifier::BOLD),
                ),
                Line::raw(mime_type.clone()),
                Line::raw("Open with b"),
            ])),
            ContentPayload::Other => Some(Paragraph::new(vec![
                Line::styled(
                    "Unknown content type.",
                    Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Line::raw("File an issue, and in the meantime open in your browser with b."),
            ])),
        }
    }
}
