use bbml::render;
use pretty_assertions::assert_eq;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

#[test]
fn test_br() {
    assert_eq!(
        render("a<br>string"),
        Paragraph::new(vec![
            vec![Span::styled("a", Style::new()),].into(),
            vec![Span::styled("string", Style::new()),].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}
#[test]
fn test_br_multiple() {
    assert_eq!(
        render("a<br><br>string"),
        Paragraph::new(vec![
            vec![Span::styled("a", Style::new()),].into(),
            vec![].into(),
            vec![Span::styled("string", Style::new()),].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}
#[test]
fn test_linebreaks() {
    assert_eq!(
        render("a\nmultiline\nstring"),
        Paragraph::new(vec![
            vec![Span::styled("a", Style::new()),].into(),
            vec![Span::styled("multiline", Style::new()),].into(),
            vec![Span::styled("string", Style::new()),].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}
