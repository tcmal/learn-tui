//! Renders [BbML](https://blackboard.github.io/rest-apis/learn/advanced/bbml) (a subset of HTML) to styled text for [`ratatui`]
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};
use tl::{Node, NodeHandle, VDom};

/// Render the given bbml as best as possible.
pub fn render(html: &str) -> Paragraph<'static> {
    let state = RenderState::new(html);
    let mut text = state.render();

    cleanup(&mut text);

    Paragraph::new(text).wrap(Wrap { trim: false })
}

/// State needed throughout the rendering process
struct RenderState<'a> {
    /// Handle into our DOM, since [`tl`] is 0-copy
    dom: VDom<'a>,
}

impl<'a> RenderState<'a> {
    /// Initialise render state with the given HTML
    fn new(html: &'a str) -> RenderState<'a> {
        let dom = tl::parse(html, tl::ParserOptions::default()).unwrap();
        Self { dom }
    }

    /// Render everything into a text object
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

    /// Actual internal rendering function
    fn render_internal(&self, text: &mut Text<'static>, handle: &NodeHandle, curr_style: Style) {
        let node = handle.get(self.dom.parser()).unwrap();
        match node {
            Node::Tag(t) => {
                let tag_name = &*t.name().as_utf8_str();
                let c = t.children();
                let children = c.top();
                match tag_name {
                    "br" => newline(text),

                    // Block text elements, which force their own line and may change the style
                    "h4" | "h5" | "h6" | "div" | "p" => {
                        let new_style = match tag_name {
                            "h4" => curr_style
                                .underline_color(Color::White)
                                .add_modifier(Modifier::BOLD),
                            "h5" | "h6" => curr_style.add_modifier(Modifier::BOLD),
                            "div" | "p" => curr_style,
                            _ => unreachable!(),
                        };

                        ensure_line_empty(text);
                        for child in children.iter() {
                            self.render_internal(text, child, new_style);
                        }
                        ensure_line_empty(text);
                    }

                    // Inline text elements, which at most change the style
                    "span" | "strong" | "em" | "a" | "li" => {
                        let new_style = match tag_name {
                            "a" => curr_style.fg(Color::Blue),
                            "strong" => curr_style.add_modifier(Modifier::BOLD),
                            "em" => curr_style.add_modifier(Modifier::ITALIC),
                            _ => curr_style,
                        };

                        for child in children.iter() {
                            self.render_internal(text, child, new_style);
                        }
                    }

                    // Lists
                    "ul" | "ol" => {
                        // Function for getting next label
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
                            // Render into new text object
                            let mut subtext = Text::raw("");
                            let child_node = child.get(self.dom.parser()).unwrap();
                            self.render_internal(&mut subtext, child, curr_style);

                            if empty_or_whitespace(&subtext) {
                                continue;
                            }

                            match child_node {
                                // Sublists don't use <li>s
                                Node::Tag(t)
                                    if t.name().as_utf8_str() == "ul"
                                        || t.name().as_utf8_str() == "ol" =>
                                {
                                    // Remove padding
                                    subtext.lines.remove(0);
                                    subtext.lines.pop();
                                    subtext.lines.pop();

                                    // Don't use label, just indent further
                                    for i in 0..subtext.lines.len() {
                                        subtext.lines[i].spans.insert(0, Span::raw("  "));
                                    }
                                }
                                _ => {
                                    // Add label at top, and indent other lines
                                    subtext.lines[0].spans.insert(0, Span::raw(next_item()));
                                    for i in 1..subtext.lines.len() {
                                        subtext.lines[i].spans.insert(0, Span::raw("    "));
                                    }
                                }
                            };

                            text.lines.extend(subtext.lines);
                        }

                        // padding
                        ensure_line_empty(text);
                        newline(text);
                    }

                    // Gracefully degrade on unknown tags
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
            // Actual text
            Node::Raw(s) => {
                let s = collapse_whitespace(&s.as_utf8_str());
                if !s.contains('\n') {
                    append(text, Span::styled(s, curr_style));
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

/// Add a newline to the text
fn newline(text: &mut Text<'static>) {
    text.lines.push(Line {
        spans: vec![],
        alignment: None,
    });
}

/// Ensure that the last line of the text is empty
fn ensure_line_empty(text: &mut Text<'static>) {
    if !currline_empty(text) {
        newline(text);
    }
}
/// Append a span to the last line of the text
fn append(text: &mut Text<'static>, span: Span<'static>) {
    match text.lines.last_mut() {
        Some(l) => l.spans.push(span),
        None => text.lines.push(span.into()),
    };
}

/// Check if the current line is empty
fn currline_empty(text: &Text<'static>) -> bool {
    text.lines.is_empty() || text.lines[text.lines.len() - 1].spans.is_empty()
}

/// Check if the given text is empty or only whitespace
fn empty_or_whitespace(text: &Text<'static>) -> bool {
    text.lines
        .iter()
        .all(|l| l.spans.iter().all(|s| s.content.is_empty()))
}

/// Collapse all whitespace in a string
fn collapse_whitespace(s: &str) -> String {
    let s = s.trim();
    let mut collapsed = String::with_capacity(s.len());
    let mut last = ' ';

    for c in s.chars() {
        if c.is_whitespace() && last.is_whitespace() {
            continue;
        }

        collapsed.push(c);
        last = c;
    }

    collapsed
}

/// Cleans up text, removing empty spans and leading/trailing lines
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
