use bbml::render;
use pretty_assertions::assert_eq;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

#[test]
fn test_ul() {
    assert_eq!(
        render("<ul><li>a</li><li>b</li><li>c</li></ul>"),
        Paragraph::new(vec![
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("a", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("b", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("c", Style::new()),
            ]
            .into(),
            vec![].into()
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_ul_multiline() {
    assert_eq!(
        render("<ul><li>a<br>long list item</li><li>b</li><li>c</li></ul>"),
        Paragraph::new(vec![
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("a", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("    ", Style::new()),
                Span::styled("long list item", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("b", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("c", Style::new()),
            ]
            .into(),
            vec![].into()
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_ol() {
    assert_eq!(
        render("<ol><li>a</li><li>b</li><li>c</li></ul>"),
        Paragraph::new(vec![
            vec![
                Span::styled("1. ", Style::new()),
                Span::styled("a", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("2. ", Style::new()),
                Span::styled("b", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("3. ", Style::new()),
                Span::styled("c", Style::new()),
            ]
            .into(),
            vec![].into()
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_ol_multiline() {
    assert_eq!(
        render("<ol><li>a<br>long list item</li><li>b</li><li>c</li></ul>"),
        Paragraph::new(vec![
            vec![
                Span::styled("1. ", Style::new()),
                Span::styled("a", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("    ", Style::new()),
                Span::styled("long list item", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("2. ", Style::new()),
                Span::styled("b", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("3. ", Style::new()),
                Span::styled("c", Style::new()),
            ]
            .into(),
            vec![].into()
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_ul_nested() {
    assert_eq!(
        render("<ul><li>a</li><ul><li>b</li></ul><li>c</li></ul>"),
        Paragraph::new(vec![
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("a", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("  ", Style::new()),
                Span::styled("  - ", Style::new()),
                Span::styled("b", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("  - ", Style::new()),
                Span::styled("c", Style::new()),
            ]
            .into(),
            vec![].into()
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_ol_nested() {
    assert_eq!(
        render("<ol><li>a</li><ol><li>b</li></ol><li>c</li></ol>"),
        Paragraph::new(vec![
            vec![
                Span::styled("1. ", Style::new()),
                Span::styled("a", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("  ", Style::new()),
                Span::styled("1. ", Style::new()),
                Span::styled("b", Style::new()),
            ]
            .into(),
            vec![
                Span::styled("2. ", Style::new()),
                Span::styled("c", Style::new()),
            ]
            .into(),
            vec![].into()
        ])
        .wrap(Wrap { trim: false })
    );
}
