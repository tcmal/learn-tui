use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Frame;

use crate::{app::App, store::Store};

mod content;
mod course;
mod courses;

pub enum ActivePage {
    Courses(courses::State),
    Course(course::State),
    Content(content::State),
}

pub enum Action {
    NewScreen(ActivePage),
    None,
}

trait Page {
    fn draw(&mut self, store: &Store, frame: &mut Frame);
    fn handle_key(&mut self, store: &Store, key: KeyEvent) -> Result<Action>;
}

impl ActivePage {
    pub fn new() -> Result<Self> {
        Ok(ActivePage::Courses(courses::State::default()))
    }
}

impl App {
    pub fn draw(&mut self, frame: &mut Frame) {
        match &mut self.curr_page {
            ActivePage::Courses(s) => s.draw(&self.store, frame),
            ActivePage::Course(s) => s.draw(&self.store, frame),
            ActivePage::Content(s) => s.draw(&self.store, frame),
        }
    }
    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            // Exit application on `Ctrl-C`
            KeyCode::Char('c') | KeyCode::Char('C') => {
                if key.modifiers == KeyModifiers::CONTROL {
                    self.quit();
                }
                Ok(())
            }
            // Other handlers you could add here.
            _ => {
                let action = match &mut self.curr_page {
                    ActivePage::Courses(s) => s.handle_key(&self.store, key),
                    ActivePage::Course(s) => s.handle_key(&self.store, key),
                    ActivePage::Content(s) => s.handle_key(&self.store, key),
                }?;
                match action {
                    Action::NewScreen(s) => self.curr_page = s,
                    Action::None => (),
                };

                Ok(())
            }
        }
    }
}
