use ratatui::{
    prelude::Rect,
    style::Stylize,
    widgets::{Paragraph, Wrap},
    Frame,
};

use crate::{
    event::Event,
    main_screen::{self, panes::Pane, Action},
    store::Store,
};

#[derive(Debug, Default)]
pub struct WelcomeViewer {}

impl Pane for WelcomeViewer {
    fn draw(&mut self, _: &Store, frame: &mut Frame, area: Rect) {
        frame.render_widget(welcome_message(), area);
    }

    fn handle_event(&mut self, _: &mut Store, _: Event) -> main_screen::Action {
        Action::None
    }
}

fn welcome_message() -> Paragraph<'static> {
    Paragraph::new(vec![
        vec!["Welcome to learn-tui!\n".blue().bold()].into(),
        vec![
            "Use ".into(),
            "j/k or ↓/↑".blue(),
            " to navigate up and down, then ".into(),
            "Enter".blue(),
            " to select an item.".into(),
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
            "Links have ".into(),
            "blue".blue(),
            " text and a number after them. Hit ".into(),
            "f".blue(),
            " then type the number to open them.".into(),
        ]
        .into(),
        vec![
            "At any point, use ".into(),
            "b".blue(),
            " to try to open the selected item in your browser, or ".into(),
            "d".blue(),
            " to try to download it.".into(),
        ]
        .into(),
        vec!["Use ".into(), "Ctrl-C".blue(), " to quit.".into()].into(),
    ])
    .wrap(Wrap { trim: false })
}
