use bbml::render;
use ratatui::{
    style::{Color, Style},
    text::Span,
    widgets::{Paragraph, Wrap},
};

#[test]
fn test_a_link() {
    let (text, links) = render("<a href=\"google.com\">a link</a>");
    assert_eq!(
        text,
        Paragraph::new(vec![vec![
            Span::styled("a link", Style::new().fg(Color::Blue)),
            Span::styled("[0]", Style::new().fg(Color::Blue))
        ]
        .into(),])
        .wrap(Wrap { trim: false })
    );

    assert_eq!(links, vec!["google.com".to_string()]);
}
