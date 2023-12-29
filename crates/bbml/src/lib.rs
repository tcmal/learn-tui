use std::iter::once;

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};
use tl::{Node, NodeHandle, VDom};

#[derive(Debug)]
pub enum RenderResult {
    Text(Text<'static>),
    Line(Line<'static>),
    Span(Span<'static>),
}

pub fn render(html: &str) -> Paragraph<'static> {
    let state = RenderState::new(html);

    let results: Vec<_> = state
        .dom
        .children()
        .iter()
        .map(|n| state.render_internal(n, Style::default()))
        .collect();

    let mut text: Text = match join_results(results.into_iter()) {
        RenderResult::Text(t) => t,
        RenderResult::Line(l) => l.into(),
        RenderResult::Span(s) => s.into(),
    };
    cleanup(&mut text);

    Paragraph::new(text).wrap(Wrap { trim: false })
}

fn cleanup(text: &mut Text) {
    text.lines
        .iter_mut()
        .for_each(|l| l.spans.retain(|s| !s.content.is_empty()));
    text.lines.retain(|l| !l.spans.is_empty());
}

struct RenderState<'a> {
    dom: VDom<'a>,
}

impl<'a> RenderState<'a> {
    pub fn new(html: &'a str) -> RenderState<'a> {
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        Self { dom }
    }

    fn render_internal(&'a self, handle: &NodeHandle, curr_style: Style) -> RenderResult {
        let node = handle.get(self.dom.parser()).unwrap();
        match node {
            Node::Tag(t) => {
                let tag_name = &*t.name().as_utf8_str();
                let c = t.children();
                let children = c.top();
                match tag_name {
                    "br" => RenderResult::Line(Line {
                        spans: vec![],
                        alignment: None,
                    }),
                    "h4" | "h5" | "h6" => {
                        let new_style = match tag_name {
                            "h4" => curr_style
                                .underline_color(Color::White)
                                .add_modifier(Modifier::BOLD),
                            "h5" | "h6" => curr_style.add_modifier(Modifier::BOLD),
                            _ => unreachable!(),
                        };
                        join_results(
                            children
                                .iter()
                                .map(|child| self.render_internal(child, new_style))
                                .chain(once(RenderResult::Line(Line {
                                    spans: vec![],
                                    alignment: None,
                                }))),
                        )
                    }
                    "div" | "p" | "span" | "strong" | "a" | "li" => {
                        let new_style = if tag_name == "a" {
                            curr_style.fg(Color::Blue)
                        } else {
                            curr_style
                        };

                        join_results(
                            children
                                .iter()
                                .map(|child| self.render_internal(child, new_style)),
                        )
                    }
                    "ul" | "ol" => {
                        let mut next_item: Box<dyn FnMut() -> String> = match tag_name {
                            "ul" => Box::new(|| "  - ".to_string()),
                            "ol" => {
                                let mut i = 0;
                                Box::new(move || {
                                    i += 1;
                                    format!("{}. ", i)
                                })
                            }
                            _ => unreachable!(),
                        };
                        join_results(
                            children
                                .iter()
                                .map(|child| self.render_internal(child, curr_style))
                                .map(|child| match child {
                                    RenderResult::Text(mut t) => {
                                        if !t.lines.is_empty() {
                                            t.lines[0].spans.insert(0, Span::raw(next_item()));
                                            for i in 1..t.lines.len() {
                                                t.lines[i].spans.insert(0, Span::raw("    "));
                                            }
                                        }
                                        RenderResult::Text(t)
                                    }
                                    RenderResult::Line(mut l) => {
                                        l.spans.insert(0, Span::raw(next_item()));
                                        RenderResult::Line(l)
                                    }
                                    RenderResult::Span(s) => {
                                        RenderResult::Line(vec![Span::raw(next_item()), s].into())
                                    }
                                }),
                        )
                    }
                    s => {
                        log::error!("unknown tag: {}", s);
                        join_results(t.children().top().iter().map(|child| {
                            self.render_internal(
                                child,
                                curr_style.fg(Color::Red).underline_color(Color::Red),
                            )
                        }))
                    }
                }
            }
            Node::Raw(s) => {
                let s = s.as_utf8_str();
                if s.contains('\n') {
                    RenderResult::Text(
                        s.split('\n')
                            .map(|l| Line::styled(l.trim().to_string(), curr_style))
                            .collect::<Vec<_>>()
                            .into(),
                    )
                } else {
                    RenderResult::Span(Span::styled(s.to_string(), curr_style))
                }
            }
            Node::Comment(_) => RenderResult::Span(Span::default()),
        }
    }
}

fn join_results(mut results: impl Iterator<Item = RenderResult>) -> RenderResult {
    let Some(mut out) = results.next() else {
        return RenderResult::Span(Span::raw(""));
    };
    for part in results {
        out = match (out, part) {
            (RenderResult::Text(mut t), RenderResult::Text(u)) => {
                t.extend(u);
                RenderResult::Text(t)
            }
            (RenderResult::Text(mut t), RenderResult::Line(l)) => {
                t.lines.push(l);
                RenderResult::Text(t)
            }
            (RenderResult::Line(l), RenderResult::Text(mut t)) => {
                t.lines.insert(0, l);
                RenderResult::Text(t)
            }
            (RenderResult::Text(mut t), RenderResult::Span(s)) => {
                match t.lines.last_mut() {
                    Some(l) => l.spans.push(s),
                    None => t.lines.push(s.into()),
                }
                RenderResult::Text(t)
            }
            (RenderResult::Span(s), RenderResult::Text(mut t)) => {
                match t.lines.first_mut() {
                    Some(l) => l.spans.insert(0, s),
                    None => t.lines.push(s.into()),
                }
                RenderResult::Text(t)
            }

            (RenderResult::Line(l), RenderResult::Line(m)) => RenderResult::Text(vec![l, m].into()),
            (RenderResult::Line(l), RenderResult::Span(s)) => {
                RenderResult::Text(vec![l, s.into()].into())
            }
            (RenderResult::Span(s), RenderResult::Line(l)) => {
                RenderResult::Text(vec![s.into(), l].into())
            }
            (RenderResult::Span(s), RenderResult::Span(m)) => RenderResult::Line(vec![s, m].into()),
        };
    }
    out
}
