use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{
    prelude::Alignment,
    widgets::{Block, Borders, List, Paragraph},
    Frame,
};

use crate::{store::Store, widgets::StatefulList};

use super::{course, Action, ActivePage, Page};

#[derive(Default)]
pub struct State {
    list: StatefulList,
}

impl Page for State {
    fn draw(&mut self, store: &Store, frame: &mut Frame) {
        let Some(courses) = store.my_courses() else {
            frame.render_widget(Paragraph::new("Loading..."), frame.size());
            return;
        };
        let items = courses.iter().map(|c| c.name.as_str());
        self.list.render_to(
            frame,
            frame.size(),
            List::new(items)
                .block(
                    Block::default()
                        .title("Courses")
                        .borders(Borders::ALL)
                        .title_alignment(Alignment::Center),
                )
                .highlight_symbol(">>"),
        );
    }

    fn handle_key(&mut self, store: &Store, key: KeyEvent) -> Result<Action> {
        match key.code {
            KeyCode::Down | KeyCode::Char('j') => {
                self.list.next();
            }
            KeyCode::Up | KeyCode::Char('k') => {
                self.list.previous();
            }
            KeyCode::Enter | KeyCode::Char('l') => {
                if let (Some(sel), Some(courses)) = (self.list.selected(), store.my_courses()) {
                    let course_id = &courses[sel].id;
                    return Ok(Action::NewScreen(ActivePage::Course(course::State::new(
                        course_id.to_string(),
                    ))));
                }
            }
            _ => (),
        };

        Ok(Action::None)
    }
}
