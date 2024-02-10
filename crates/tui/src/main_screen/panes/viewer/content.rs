use crossterm::event::{KeyCode, KeyModifiers};
use edlearn_client::content::ContentPayload;
use log::debug;
use ratatui::{
    prelude::Margin,
    style::{Color, Modifier, Style, Stylize},
    text::Line,
    widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState},
};

use crate::{
    event::Event,
    main_screen::{panes::Pane, Action},
    store::{ContentIdx, DownloadState, Store},
    styles::error_text,
};

pub struct ContentViewer {
    content_idx: ContentIdx,

    /// Scroll status
    y_offset: u16,
    jump_y_offset: u16,

    /// A cached render of what we're displaying, to avoid constantly re-rendering.
    cached_render: Option<Paragraph<'static>>,

    /// A list of links we're displaying. The user can specify an index to visit them
    displayed_links: Vec<String>,

    /// State for link entry
    link_idx_max_digits: usize,
    link_entry_acc: usize,
    link_entry_digits: Option<usize>,
}
impl ContentViewer {
    pub(crate) fn new(content_idx: ContentIdx) -> ContentViewer {
        Self {
            content_idx,
            y_offset: 0,
            jump_y_offset: 0,
            cached_render: None,
            displayed_links: vec![],
            link_idx_max_digits: 0,
            link_entry_acc: 0,
            link_entry_digits: None,
        }
    }

    /// Render the referenced content item, if it is loaded
    fn render_content(&mut self, store: &Store) -> Paragraph<'static> {
        let content = store.content(self.content_idx);
        match &content.payload {
            ContentPayload::Page => {
                let Some(text) = store.page_text(self.content_idx) else {
                    store.request_page_text(self.content_idx);
                    return Paragraph::new("Loading...");
                };
                let (text, links) = bbml::render(text);
                self.set_displayed_links(links);
                self.cached_render = Some(text);
                self.cached_render.clone().unwrap()
            }
            ContentPayload::Link(l) => {
                self.cached_render = Some(Paragraph::new(format!("Link to {}. Open with b", l)));
                self.cached_render.clone().unwrap()
            }
            ContentPayload::Folder => {
                self.cached_render = Some(Paragraph::new("Folder"));
                self.cached_render.clone().unwrap()
            }
            ContentPayload::File {
                file_name,
                mime_type,
                ..
            } => {
                let mut ls = vec![
                    file_name.to_string().blue().bold().into(),
                    Line::raw(mime_type.clone()),
                    Line::raw("Open with b"),
                ];
                if let Some((req, state)) = store.download_status(self.content_idx) {
                    match state {
                        DownloadState::Queued => ls.push(Line::styled(
                            "Queued for download",
                            Style::new().fg(Color::Gray),
                        )),
                        DownloadState::InProgress(p) => ls.push(Line::styled(
                            format!("Downloading - {:.2}%", p * 100.0),
                            Style::new().fg(Color::Blue),
                        )),
                        DownloadState::Completed => ls.push(Line::styled(
                            format!("Downloaded to {}. Press o to open.", req.dest),
                            Style::new().fg(Color::Green),
                        )),
                        DownloadState::Errored(e) => ls.extend(error_text(e.to_string()).lines),
                    }
                } else {
                    self.cached_render = Some(Paragraph::new(ls.clone()));
                }
                Paragraph::new(ls)
            }
            ContentPayload::Other => {
                self.cached_render = Some(Paragraph::new(vec![
                    Line::styled(
                        "Unknown content type.",
                        Style::new().fg(Color::Red).add_modifier(Modifier::BOLD),
                    ),
                    Line::raw("File an issue, and in the meantime open in your browser with b."),
                ]));
                self.cached_render.clone().unwrap()
            }
        }
    }

    fn set_displayed_links(&mut self, links: Vec<String>) {
        self.link_idx_max_digits = if !links.is_empty() {
            links.len().ilog10() as usize + 1
        } else {
            0
        };
        self.displayed_links = links;
        self.link_entry_acc = 0;
        self.link_entry_digits = None;
        debug!(
            "displaying {} links (max digits = {})",
            self.displayed_links.len(),
            self.link_idx_max_digits
        );
    }

    fn open_referenced_link(&mut self) -> Action {
        let Some(href) = self.displayed_links.get(self.link_entry_acc) else {
            return Action::Flash(error_text("No link found".to_string()));
        };

        if let Err(e) = open::that(href) {
            return Action::Flash(error_text(format!("Error opening in browser: {e}")));
        }

        self.link_entry_acc = 0;
        self.link_entry_digits = None;

        Action::Flash(format!("Opened {href} in browser").into())
    }
}

impl Pane for ContentViewer {
    fn draw(
        &mut self,
        store: &crate::store::Store,
        frame: &mut ratatui::Frame,
        area: ratatui::prelude::Rect,
    ) {
        let rendered = self.render_content(store);

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

    fn handle_event(
        &mut self,
        store: &mut crate::store::Store,
        event: crate::event::Event,
    ) -> crate::main_screen::Action {
        let Event::Key(key) = event else {
            return Action::None;
        };

        match key.code {
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

            // Open in browser / open downloaded file
            KeyCode::Char('b') => {
                self.link_entry_digits = None;
                let content = store.content(self.content_idx);
                if let Err(e) = open::that(content.browser_link()) {
                    return Action::Flash(error_text(format!("Error opening in browser: {e}")));
                }
            }
            KeyCode::Char('o') => {
                self.link_entry_digits = None;
                if let Some((req, DownloadState::Completed)) =
                    store.download_status(self.content_idx)
                {
                    if let Err(e) = open::that(&req.dest) {
                        return Action::Flash(error_text(format!("Error opening file: {e}")));
                    }
                }
            }

            // Queue download
            KeyCode::Char('d') => {
                store.download_content(self.content_idx);
                self.cached_render = None;
                return Action::Flash("Queued for download".into());
            }

            // Link index entry
            KeyCode::Char('f') => {
                if self.link_idx_max_digits > 0 {
                    self.link_entry_acc = 0;
                    self.link_entry_digits = Some(0);

                    return Action::Flash(
                        "Go to... (type the number after the link)"
                            .to_string()
                            .into(),
                    );
                }
            }
            KeyCode::Enter if self.link_entry_digits.is_some() => {
                return self.open_referenced_link();
            }

            KeyCode::Char(n) if n.is_ascii_digit() => {
                if let Some(idx) = self.link_entry_digits.as_mut() {
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
                        return self.open_referenced_link();
                    } else {
                        return Action::Flash(
                            format!(
                                "Go to... {} (RET to open, or keep typing numbers)",
                                self.link_entry_acc
                            )
                            .into(),
                        );
                    }
                }
            }

            _ => (),
        };

        // Every branch where we do more digit entry returns, so if we've stopped doing that then exit that mode
        self.link_entry_digits = None;

        Action::None
    }
}
