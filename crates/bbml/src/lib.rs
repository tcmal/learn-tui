//! Renders [BbML](https://blackboard.github.io/rest-apis/learn/advanced/bbml) (a subset of HTML) to styled text for [`ratatui`]
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};
use tl::{Node, NodeHandle, VDom};

/// Render the given bbml as best as possible.
/// Returns the rendered text as a paragraph, and a list of links inside that text
pub fn render(html: &str) -> (Paragraph<'static>, Vec<String>) {
    let mut state = RenderState::new(html);
    let (mut text, links) = state.render();

    cleanup(&mut text);

    (Paragraph::new(text).wrap(Wrap { trim: false }), links)
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
    fn render(&mut self) -> (Text<'static>, Vec<String>) {
        let mut text = Text {
            lines: vec![Line {
                spans: vec![],
                alignment: None,
            }],
        };
        let mut links = vec![];
        let mut out = RenderOutput::new(&mut text, &mut links);

        for child in self.dom.children() {
            self.render_internal(&mut out, child, Style::default());
        }

        (text, links)
    }

    /// Actual internal rendering function
    fn render_internal(&self, out: &mut RenderOutput, handle: &NodeHandle, curr_style: Style) {
        let node = handle.get(self.dom.parser()).unwrap();
        match node {
            Node::Tag(t) => {
                let tag_name = &*t.name().as_utf8_str();
                let c = t.children();
                let children = c.top();
                match tag_name {
                    "br" => out.newline(),

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

                        out.ensure_line_empty();
                        for child in children.iter() {
                            self.render_internal(out, child, new_style);
                        }
                        out.ensure_line_empty();
                    }

                    // Inline text elements, which at most change the style
                    "span" | "strong" | "em" | "li" => {
                        let new_style = match tag_name {
                            "strong" => curr_style.add_modifier(Modifier::BOLD),
                            "em" => curr_style.add_modifier(Modifier::ITALIC),
                            _ => curr_style,
                        };

                        for child in children.iter() {
                            self.render_internal(out, child, new_style);
                        }
                    }

                    // Links
                    "a" => {
                        let new_style = curr_style.fg(Color::Blue);
                        for child in children.iter() {
                            self.render_internal(out, child, new_style);
                        }
                        if let Some(Some(b)) = t.attributes().get("href") {
                            let href = b.as_utf8_str().to_string();
                            let idx = out.add_link(href);

                            out.append(Span::styled(format!("[{idx}]"), new_style));
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
                            let mut suboutp = out.with_subtext(&mut subtext);
                            let child_node = child.get(self.dom.parser()).unwrap();
                            self.render_internal(&mut suboutp, child, curr_style);

                            if suboutp.empty_or_whitespace() {
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

                            out.text.lines.extend(subtext.lines);
                        }

                        // padding
                        out.ensure_line_empty();
                        out.newline();
                    }

                    // Gracefully degrade on unknown tags
                    s => {
                        log::error!("unknown tag: {}", s);
                        t.children().top().iter().for_each(|child| {
                            self.render_internal(
                                out,
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
                    out.append(Span::styled(s, curr_style));
                } else {
                    for l in s.split('\n') {
                        out.append(Span::styled(l.to_string(), curr_style));
                        out.newline();
                    }
                }
            }
            Node::Comment(_) => (),
        }
    }
}

struct RenderOutput<'a> {
    text: &'a mut Text<'static>,
    links: &'a mut Vec<String>,
}

impl<'a> RenderOutput<'a> {
    fn new(text: &'a mut Text<'static>, links: &'a mut Vec<String>) -> Self {
        Self { text, links }
    }

    /// Add a newline to the text
    fn newline(&mut self) {
        self.text.lines.push(Line {
            spans: vec![],
            alignment: None,
        });
    }

    /// Ensure that the last line of the text is empty
    fn ensure_line_empty(&mut self) {
        if !self.currline_empty() {
            self.newline();
        }
    }
    /// Append a span to the last line of the text
    fn append(&mut self, span: Span<'static>) {
        match self.text.lines.last_mut() {
            Some(l) => l.spans.push(span),
            None => self.text.lines.push(span.into()),
        };
    }

    /// Check if the current line is empty
    fn currline_empty(&mut self) -> bool {
        self.text.lines.is_empty() || self.text.lines[self.text.lines.len() - 1].spans.is_empty()
    }

    /// Check if the given text is empty or only whitespace
    fn empty_or_whitespace(&mut self) -> bool {
        self.text
            .lines
            .iter()
            .all(|l| l.spans.iter().all(|s| s.content.is_empty()))
    }

    /// Add a link to the encountered list, returning its index
    fn add_link(&mut self, href: String) -> usize {
        self.links.push(href);
        self.links.len() - 1
    }

    fn with_subtext<'b>(&'b mut self, subtext: &'b mut Text<'static>) -> RenderOutput<'b>
    where
        'a: 'b,
    {
        RenderOutput {
            text: subtext,
            links: self.links,
        }
    }
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
fn cleanup(text: &mut Text<'static>) {
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
