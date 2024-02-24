use bbml::render;
use pretty_assertions::assert_eq;
use ratatui::{
    prelude::*,
    widgets::{Paragraph, Wrap},
};

#[test]
fn test_table_small() {
    assert_eq!(
        dbg!(render("<table><tr><td>1</td><td>2</td><td>3</td></tr><tr><td>4</td><td>5</td><td>6</td></tr><tr><td>7</td><td>8</td><td>9</td></tr></table>").0),
        Paragraph::new(vec![
            vec![Span::raw("┌─┬─┬─┐")].into(),
            vec![Span::raw("│"), Span::raw("1"), Span::raw("│"), Span::raw("2"), Span::raw("│"), Span::raw("3"), Span::raw("│")].into(),
            vec![Span::raw("├─┼─┼─┤")].into(),
            vec![Span::raw("│"), Span::raw("4"), Span::raw("│"), Span::raw("5"), Span::raw("│"), Span::raw("6"), Span::raw("│")].into(),
            vec![Span::raw("├─┼─┼─┤")].into(),
            vec![Span::raw("│"), Span::raw("7"), Span::raw("│"), Span::raw("8"), Span::raw("│"), Span::raw("9"), Span::raw("│")].into(),
            vec![Span::raw("└─┴─┴─┘")].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_table_var_col_widths() {
    assert_eq!(
        dbg!(
            render(
                "<table>
<tr><td>aaa</td><td>a</td><td>a</td></tr>
<tr><td>b</td><td>bbb</td><td>b</td></tr>
<tr><td>c</td><td>c</td><td>ccc</td></tr>
</table>"
            )
            .0
        ),
        Paragraph::new(vec![
            vec![Span::raw("┌───┬───┬───┐")].into(),
            vec![
                Span::raw("│"),
                Span::raw("aaa"),
                Span::raw("│"),
                Span::raw("a"),
                Span::raw("  "),
                Span::raw("│"),
                Span::raw("a"),
                Span::raw("  "),
                Span::raw("│")
            ]
            .into(),
            vec![Span::raw("├───┼───┼───┤")].into(),
            vec![
                Span::raw("│"),
                Span::raw("b"),
                Span::raw("  "),
                Span::raw("│"),
                Span::raw("bbb"),
                Span::raw("│"),
                Span::raw("b"),
                Span::raw("  "),
                Span::raw("│")
            ]
            .into(),
            vec![Span::raw("├───┼───┼───┤")].into(),
            vec![
                Span::raw("│"),
                Span::raw("c"),
                Span::raw("  "),
                Span::raw("│"),
                Span::raw("c"),
                Span::raw("  "),
                Span::raw("│"),
                Span::raw("ccc"),
                Span::raw("│")
            ]
            .into(),
            vec![Span::raw("└───┴───┴───┘")].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_table_descends_thead_tbody() {
    assert_eq!(
        dbg!(render("<table><thead><tr><td>1</td><td>2</td><td>3</td></tr></thead><tbody><tr><td>4</td><td>5</td><td>6</td></tr><tr><td>7</td><td>8</td><td>9</td></tr></tbody></table>").0),
        Paragraph::new(vec![
            vec![Span::raw("┌─┬─┬─┐")].into(),
            vec![Span::raw("│"), Span::raw("1"), Span::raw("│"), Span::raw("2"), Span::raw("│"), Span::raw("3"), Span::raw("│")].into(),
            vec![Span::raw("├─┼─┼─┤")].into(),
            vec![Span::raw("│"), Span::raw("4"), Span::raw("│"), Span::raw("5"), Span::raw("│"), Span::raw("6"), Span::raw("│")].into(),
            vec![Span::raw("├─┼─┼─┤")].into(),
            vec![Span::raw("│"), Span::raw("7"), Span::raw("│"), Span::raw("8"), Span::raw("│"), Span::raw("9"), Span::raw("│")].into(),
            vec![Span::raw("└─┴─┴─┘")].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_table_imitate_margin_collapse() {
    assert_eq!(
        dbg!(render("<table><tr><td><p>1</p></td><td><p>2</p></td><td><p>3</p></td></tr><tr><td><p>4</p></td><td><p>5</p></td><td><p>6</p></td></tr><tr><td><p>7</p></td><td><p>8</p></td><td><p>9</p></td></tr></table>").0),
        Paragraph::new(vec![
            vec![Span::raw("┌─┬─┬─┐")].into(),
            vec![Span::raw("│"), Span::raw("1"), Span::raw("│"), Span::raw("2"), Span::raw("│"), Span::raw("3"), Span::raw("│")].into(),
            vec![Span::raw("├─┼─┼─┤")].into(),
            vec![Span::raw("│"), Span::raw("4"), Span::raw("│"), Span::raw("5"), Span::raw("│"), Span::raw("6"), Span::raw("│")].into(),
            vec![Span::raw("├─┼─┼─┤")].into(),
            vec![Span::raw("│"), Span::raw("7"), Span::raw("│"), Span::raw("8"), Span::raw("│"), Span::raw("9"), Span::raw("│")].into(),
            vec![Span::raw("└─┴─┴─┘")].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}

#[test]
fn test_table_width_wraps_properly() {
    assert_eq!(
        dbg!(render("<table><tr><td>11111111111111111111111111111111111111111111111111111111111111111111111</td></tr></table>").0),
        Paragraph::new(vec![
            vec![Span::raw("┌────────────────────────────────────────────────────────────────────┐")].into(),
            vec![Span::raw("│"), Span::raw("11111111111111111111111111111111111111111111111111111111111111111111"), Span::raw("│")].into(),
            vec![Span::raw("│"), Span::raw("111"), Span::raw("                                                                 "), Span::raw("│")].into(),
            vec![Span::raw("└────────────────────────────────────────────────────────────────────┘")].into(),
        ])
        .wrap(Wrap { trim: false })
    );
}
