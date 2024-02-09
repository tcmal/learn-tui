use crossterm::event::{KeyCode, KeyModifiers};
use edlearn_client::content::ContentPayload;
use log::debug;
use ratatui::{
    prelude::{Margin, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Text},
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
    Frame,
};

use crate::{
    event::Event,
    store::{ContentIdx, DownloadState, Store},
};

use super::{Action, Pane};

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

/// Shows [`Document`]s to the user, and provides scrolling, etc.
#[derive(Default)]
pub struct Viewer {
    /// The thing we're currently showing.
    show: Document,

    /// Scroll status
    y_offset: u16,
    jump_y_offset: u16,

    /// A cached render of what we're displaying, to avoid constantly re-creating.
    cached_render: Option<Paragraph<'static>>,

    /// A list of links we're displaying. The user can specify an index to visit them
    displayed_links: Vec<String>,

    link_idx_max_digits: usize,
    link_entry_acc: usize,
    link_entry_digits: Option<usize>,
}

impl Viewer {
    /// Set the content that we will show from next draw.
    pub fn show(&mut self, d: Document) {
        self.show = d;
        self.y_offset = 0;
        self.cached_render = None;
        self.displayed_links = vec![];
    }

    /// Render the current document, updating the render cache if necessary
    fn render(&mut self, store: &Store) -> Paragraph<'static> {
        if let Some(p) = self.cached_render.clone() {
            return p;
        }

        match self.show {
            Document::Content(idx) => {
                if let Some(p) = self.render_content(&store, idx) {
                    // Cache rendered content
                    self.cached_render = Some(p.clone());
                    p
                } else {
                    // Don't cache loading screen, so we re-render once it loads
                    Paragraph::new("Loading...")
                }
            }
            Document::Welcome => {
                // Static welcome message
                let p = welcome_message();
                self.cached_render = Some(p.clone());
                p
            }
            // Build downloads page (not cached)
            Document::Downloads => Paragraph::new(
                store
                    .download_queue()
                    .flat_map(|(req, state)| {
                        vec![
                            vec![
                                req.orig_filename.to_string().blue(),
                                match &state {
                                    DownloadState::Queued => " - Queued".gray(),
                                    DownloadState::InProgress(p) => {
                                        format!(" - {:.2}%", p * 100.0).blue()
                                    }
                                    DownloadState::Completed => " - Completed".green(),
                                    DownloadState::Errored(e) => format!(" - {e}").red(),
                                },
                            ]
                            .into(),
                            vec![req.dest.to_string().gray()].into(),
                        ]
                    })
                    .collect::<Vec<Line>>(),
            ),
        }
    }

    /// Render the referenced content item, if it is loaded
    fn render_content(
        &mut self,
        store: &Store,
        content_idx: ContentIdx,
    ) -> Option<Paragraph<'static>> {
        let content = store.content(content_idx);
        match &content.payload {
            ContentPayload::Page => {
                let Some(text) = store.page_text(content_idx) else {
                    store.request_page_text(content_idx);
                    return None;
                };
                let (text, links) = bbml::render(text);
                self.set_displayed_links(links);
                Some(text)
            }
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

    fn set_displayed_links(&mut self, links: Vec<String>) {
        self.link_idx_max_digits = if !links.is_empty() {
            links.len().ilog10() as usize + 1
        } else {
            0
        };
        self.displayed_links = links;
        debug!(
            "displaying {} links (max digits = {})",
            self.displayed_links.len(),
            self.link_idx_max_digits
        );
    }
}

impl Pane for Viewer {
    fn draw(&mut self, store: &Store, frame: &mut Frame, area: Rect) {
        let rendered = self.render(store);

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

    fn handle_event(&mut self, store: &mut Store, event: Event) -> Action {
        let Event::Key(key) = event else {
            return Action::None;
        };

        match key.code {
            // Exit
            KeyCode::Char('q') | KeyCode::Esc => return Action::FocusNavigation,

            // Basic vim-like navigation
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

            // Open in browser
            KeyCode::Char('b') => {
                if let Document::Content(content_idx) = self.show {
                    let content = store.content(content_idx);
                    if let Err(e) = open::that(content.browser_link()) {
                        todo!("deal with open error {}", e);
                    }
                };
            }

            // Queue download
            KeyCode::Char('d') => {
                if let Document::Content(content_idx) = self.show {
                    store.download_content(content_idx);
                };
            }

            // Link index entry
            KeyCode::Char('f') => {
                if self.link_idx_max_digits > 0 {
                    self.link_entry_acc = 0;
                    self.link_entry_digits = Some(0);
                }
            }
            KeyCode::Char(n) if n.is_digit(10) => match self.link_entry_digits.as_mut() {
                Some(idx) => {
                    // add new digit to end of number
                    self.link_entry_acc *= 10;
                    self.link_entry_acc += n.to_digit(10).unwrap() as usize;
                    *idx += 1;

                    // check if done entering
                    debug!(
                        "entered {idx} digits / {}. acc = {}",
                        self.link_idx_max_digits, self.link_entry_acc
                    );
                    if *idx == self.link_idx_max_digits {
                        let Some(href) = self.displayed_links.get(self.link_entry_acc) else {
                            debug!("invalid idx");
                            return Action::None; // TODO: show this somehow
                        };

                        if let Err(_) = open::that(href) {
                            todo!("deal with open error");
                        }

                        self.link_entry_acc = 0;
                        self.link_entry_digits = None;
                    }
                }
                None => (),
            },

            _ => (),
        };

        Action::None
    }
}

fn welcome_message() -> Paragraph<'static> {
    Paragraph::new(Into::<Text>::into(vec![
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
            "When an item is selected, you can scroll the viewer pane using ".into(),
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
    .wrap(Wrap { trim: false })
}
