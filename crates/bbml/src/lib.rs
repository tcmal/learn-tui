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
    let mut text = state.render();

    cleanup(&mut text);

    Paragraph::new(text).wrap(Wrap { trim: false })
}

fn cleanup(text: &mut Text) {
    text.lines
        .iter_mut()
        .for_each(|l| l.spans.retain(|s| !s.content.is_empty()));
    if !text.lines.is_empty() && text.lines[0].spans.is_empty() {
        text.lines.remove(0);
    }

    if !text.lines.is_empty() && text.lines.last().unwrap().spans.is_empty() {
        text.lines.remove(text.lines.len() - 1);
    }
}

struct RenderState<'a> {
    dom: VDom<'a>,
}

impl<'a> RenderState<'a> {
    pub fn new(html: &'a str) -> RenderState<'a> {
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        Self { dom }
    }
    fn render(&self) -> Text<'static> {
        let mut text = Text {
            lines: vec![Line {
                spans: vec![],
                alignment: None,
            }],
        };
        for child in self.dom.children() {
            self.render_internal(&mut text, child, Style::default());
        }

        text
    }

    fn render_internal(&self, text: &mut Text<'static>, handle: &NodeHandle, curr_style: Style) {
        let node = handle.get(self.dom.parser()).unwrap();
        match node {
            Node::Tag(t) => {
                let tag_name = &*t.name().as_utf8_str();
                let c = t.children();
                let children = c.top();
                match tag_name {
                    "br" => newline(text),
                    "h4" | "h5" | "h6" => {
                        let new_style = match tag_name {
                            "h4" => curr_style
                                .underline_color(Color::White)
                                .add_modifier(Modifier::BOLD),
                            "h5" | "h6" => curr_style.add_modifier(Modifier::BOLD),
                            _ => unreachable!(),
                        };
                        for child in children.iter() {
                            self.render_internal(text, child, new_style);
                        }
                        newline(text);
                    }
                    "span" | "strong" | "a" | "li" => {
                        let new_style = match tag_name {
                            "a" => curr_style.fg(Color::Blue),
                            "strong" => curr_style.add_modifier(Modifier::BOLD),
                            _ => curr_style,
                        };

                        for child in children.iter() {
                            self.render_internal(text, child, new_style);
                        }
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
                        for child in children.iter() {
                            let mut subtext = Text::raw("");
                            self.render_internal(&mut subtext, child, curr_style);
                            if !subtext.lines.is_empty() {
                                subtext.lines[0].spans.insert(0, Span::raw(next_item()));
                                for i in 1..subtext.lines.len() {
                                    subtext.lines[i].spans.insert(0, Span::raw("    "));
                                }
                            }

                            text.lines.extend(subtext.lines);
                        }
                    }
                    s => {
                        log::error!("unknown tag: {}", s);
                        t.children().top().iter().for_each(|child| {
                            self.render_internal(
                                text,
                                child,
                                curr_style.fg(Color::Red).underline_color(Color::Red),
                            )
                        })
                    }
                }
            }
            Node::Raw(s) => {
                let s = s.as_utf8_str();
                if !s.contains('\n') {
                    append(text, Span::styled(s.to_string(), curr_style));
                } else {
                    for l in s.split('\n') {
                        append(text, Span::styled(l.to_string(), curr_style));
                        newline(text);
                    }
                }
            }
            Node::Comment(_) => (),
        }
    }
}

fn newline(text: &mut Text<'static>) {
    text.lines.push(Line {
        spans: vec![],
        alignment: None,
    });
}

fn append(text: &mut Text<'static>, span: Span<'static>) {
    text.lines.last_mut().unwrap().spans.push(span);
}
