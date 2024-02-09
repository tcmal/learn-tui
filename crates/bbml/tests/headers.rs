use bbml::render;
use pretty_assertions::assert_eq;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

#[test]
fn test_h4() {
    assert_eq!(
        render("<h4>header</h4>").0,
        Paragraph::new(vec![vec![Span::styled(
            "header",
            Style::new().bold().underline_color(Color::White)
        )]
        .into(),])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_h5() {
    assert_eq!(
        render("<h5>header</h5>").0,
        Paragraph::new(vec![
            vec![Span::styled("header", Style::new().bold())].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}
#[test]
fn test_h6() {
    assert_eq!(
        render("<h5>header</h5>").0,
        Paragraph::new(vec![
            vec![Span::styled("header", Style::new().bold())].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}
