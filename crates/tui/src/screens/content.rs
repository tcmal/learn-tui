use std::cell::OnceCell;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::debug;
use ratatui::{widgets::Paragraph, Frame};

use crate::store::{ContentIdx, CourseId, Store};

use super::{course, Action, ActivePage, Page};

pub struct State {
    course_id: CourseId,
    content_idx: ContentIdx,
    cached_render: OnceCell<Paragraph<'static>>,
}
impl State {
    pub fn new(course_id: CourseId, content_idx: ContentIdx) -> State {
        Self {
            course_id,
            content_idx,
            cached_render: Default::default(),
        }
    }
}

impl Page for State {
    fn draw(&mut self, store: &Store, frame: &mut Frame) {
        let content = store.content(self.content_idx);
        let rendered = self.cached_render.get_or_init(|| {
            let body = &content
                .body
                .as_deref()
                .unwrap_or("<h4>nothing here...</h4>");
            debug!("rendering body: {}", body);
            bbml::render(body)
        });

        frame.render_widget(rendered.clone(), frame.size());
    }

    fn handle_key(&mut self, store: &Store, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Esc | KeyCode::Char('q') => {
                return Ok(Action::NewScreen(ActivePage::Course(course::State::new(
                    self.course_id.clone(),
                ))));
            }
            _ => (),
        }
        Ok(Action::None)
    }
}
