use ratatui::{
    prelude::Text,
    style::{Color, Style},
};

pub fn error_text(t: impl Into<Text<'static>>) -> Text<'static> {
    let mut t = t.into();
    t.patch_style(Style::default().fg(Color::Red));
    t
}
