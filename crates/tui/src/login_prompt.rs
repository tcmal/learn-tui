use crate::{auth_cache::LoginDetails, Screen};
use anyhow::Result;

#[derive(Default)]
pub struct LoginPrompt;

impl LoginPrompt {
    fn extract_details(self) -> Result<LoginDetails> {
        todo!()
    }
}

impl Screen for LoginPrompt {
    fn draw(&mut self, frame: &mut ratatui::Frame) {
        todo!()
    }

    fn handle_event(&mut self, event: crate::event::Event) -> anyhow::Result<()> {
        todo!()
    }

    fn running(&self) -> bool {
        todo!()
    }
}
