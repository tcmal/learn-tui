//! Renders [BbML](https://blackboard.github.io/rest-apis/learn/advanced/bbml) (a subset of HTML) to styled text for [`ratatui`]
use log::debug;
use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Paragraph, Wrap},
};
use tl::{HTMLTag, Node, NodeHandle, VDom};

const SCREEN_WIDTH: usize = 70;
const TABLE_VERTICAL_BORDER: char = '─';
const TABLE_MID_LEFT_BORDER: char = '├';
const TABLE_MID_INTERSECT: char = '┼';
const TABLE_MID_RIGHT_BORDER: char = '┤';
const TABLE_TOP_LEFT_BORDER: char = '┌';
const TABLE_TOP_INTERSECT: char = '┬';
const TABLE_BOT_INTERSECT: char = '┴';
const TABLE_TOP_RIGHT_BORDER: char = '┐';
const TABLE_BOT_LEFT_BORDER: char = '└';
const TABLE_BOT_RIGHT_BORDER: char = '┘';
const TABLE_HORIZ_BORDER: char = '│';

/// Render the given bbml as best as possible.
/// Returns the rendered text as a paragraph, and a list of links inside that text
pub fn render(html: &str) -> (Paragraph<'static>, Vec<String>) {
    let mut state = RenderState::new(&html);
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
                    // td is here because we deal with it at the tr level (see further down)
                    "span" | "strong" | "em" | "li" | "td" | "th" => {
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

                    // Tables
                    "table" => {
                        // Render each cell
                        let mut subtexts: Vec<Vec<Text<'static>>> = vec![];
                        self.render_table_cells(out, t, &mut subtexts);

                        debug!("{:?}", subtexts);

                        // Ensure table is a square
                        let max_cols = subtexts.iter().map(Vec::len).max().unwrap_or(0);
                        subtexts
                            .iter_mut()
                            .for_each(|v| v.resize(max_cols, "".into()));

                        // Figure out the dimensions of everything
                        let mut col_widths = (0..max_cols)
                            .map(|col_idx| {
                                subtexts
                                    .iter()
                                    .map(|r| &r[col_idx])
                                    .map(|t| t.width())
                                    .max()
                                    .unwrap_or(0)
                            })
                            .collect::<Vec<_>>();

                        let total_width = col_widths.iter().sum::<usize>() + col_widths.len() + 1;
                        let (widest_col_idx, &max_width) = col_widths
                            .iter()
                            .enumerate()
                            .max_by_key(|(_, w)| **w)
                            .unwrap_or((0, &0));
                        // Attempt to shrink largest column if we need to
                        if total_width > SCREEN_WIDTH && max_width > (total_width - SCREEN_WIDTH) {
                            let new_width = max_width - (total_width - SCREEN_WIDTH);
                            col_widths[widest_col_idx] = new_width;

                            for row in subtexts.iter_mut() {
                                wrap_text_to_width(&mut row[widest_col_idx], new_width);
                            }
                        }

                        let row_heights = subtexts
                            .iter()
                            .map(|row| row.iter().map(|cell| cell.height()).max().unwrap_or(0))
                            .collect::<Vec<_>>();

                        // Now we can output our table with the right dimensions
                        out.ensure_line_empty();

                        out.append(table_vertical_border(
                            &col_widths,
                            TABLE_TOP_LEFT_BORDER,
                            TABLE_VERTICAL_BORDER,
                            TABLE_TOP_INTERSECT,
                            TABLE_TOP_RIGHT_BORDER,
                        ));
                        let n_rows = subtexts.len();
                        for (row_idx, row) in subtexts.into_iter().enumerate() {
                            // append however many lines in this row to work with
                            let row_height = row_heights[row_idx];
                            let row_start_idx = out.text.lines.len();
                            (0..row_height).for_each(|_| {
                                out.text.lines.push(TABLE_HORIZ_BORDER.to_string().into())
                            });

                            for (col_idx, cell) in row.into_iter().enumerate() {
                                let col_width = col_widths[col_idx];
                                let added_to_lines = cell.lines.len();

                                // add to the end of the existing lines, padding if needed
                                for (line_idx, line) in cell.lines.into_iter().enumerate() {
                                    let adding_width = line.width();
                                    let add_to_line = &mut out.text.lines[row_start_idx + line_idx];
                                    add_to_line.spans.extend(line.spans);
                                    if adding_width < col_width {
                                        add_to_line
                                            .spans
                                            .push(" ".repeat(col_width - adding_width).into());
                                    }
                                }

                                // add space to the missing lines if needed
                                for i in added_to_lines..row_height {
                                    out.text.lines[row_start_idx + i]
                                        .spans
                                        .push(" ".repeat(col_width).into());
                                }

                                // add right borders
                                (0..row_height).for_each(|i| {
                                    out.text.lines[row_start_idx + i]
                                        .spans
                                        .push(TABLE_HORIZ_BORDER.to_string().into())
                                });
                            }

                            if row_idx < n_rows - 1 {
                                out.ensure_line_empty();
                                out.append(table_vertical_border(
                                    &col_widths,
                                    TABLE_MID_LEFT_BORDER,
                                    TABLE_VERTICAL_BORDER,
                                    TABLE_MID_INTERSECT,
                                    TABLE_MID_RIGHT_BORDER,
                                ));
                            }
                        }

                        out.ensure_line_empty();
                        out.append(table_vertical_border(
                            &col_widths,
                            TABLE_BOT_LEFT_BORDER,
                            TABLE_VERTICAL_BORDER,
                            TABLE_BOT_INTERSECT,
                            TABLE_BOT_RIGHT_BORDER,
                        ));
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
                let mut text = String::with_capacity(s.as_utf8_str().len());
                html_escape::decode_html_entities_to_string(
                    collapse_whitespace(&s.as_utf8_str()),
                    &mut text,
                );
                if !text.contains('\n') {
                    out.append(Span::styled(text, curr_style));
                } else {
                    for l in text.split('\n') {
                        out.append(Span::styled(l.to_string(), curr_style));
                        out.newline();
                    }
                }
            }
            Node::Comment(_) => (),
        }
    }

    fn render_table_cells(
        &self,
        out: &mut RenderOutput<'_>,
        table: &HTMLTag<'_>,
        cells: &mut Vec<Vec<Text<'static>>>,
    ) {
        for row_handle in table.children().top().iter() {
            match row_handle.get(self.dom.parser()).unwrap() {
                Node::Tag(row) => match &*row.name().as_utf8_str() {
                    "thead" | "tbody" => {
                        self.render_table_cells(out, row, cells);
                    }
                    _ => {
                        let mut cols = vec![];
                        for cell in row.children().top().iter() {
                            let mut subtext = Text::default();
                            let mut suboutp = out.with_subtext(&mut subtext);
                            self.render_internal(&mut suboutp, cell, Style::new());

                            if subtext.width() == 0 || subtext.height() == 0 {
                                continue;
                            }
                            cleanup(&mut subtext);
                            cols.push(subtext);
                        }
                        if !cols.is_empty() {
                            cells.push(cols);
                        }
                    }
                },
                _ => (),
            };
        }
    }
}

fn wrap_text_to_width(text: &mut Text<'_>, new_width: usize) {
    let mut i = 0;
    while i < text.lines.len() {
        if text.lines[i].width() > new_width {
            let new_line = chop_after(&mut text.lines[i], new_width);
            text.lines.insert(i + 1, new_line);
        } else {
            i += 1;
        }
    }
}

fn chop_after<'a>(line: &mut Line<'a>, width: usize) -> Line<'a> {
    let mut cum_width = 0;
    for i in 0..line.spans.len() {
        if cum_width + line.spans[i].width() > width {
            // split current span
            let keep = width - cum_width;
            let content = line.spans[i].content.to_owned();
            line.spans[i].content = content.chars().take(keep).collect::<String>().into();

            let mut new_line = vec![Span::styled(
                content.chars().skip(keep).collect::<String>(),
                line.spans[i].style,
            )];
            line.spans.drain(i + 1..).for_each(|s| new_line.push(s));
            return new_line.into();
        } else {
            cum_width += line.spans[i].width();
        }
    }
    vec![].into()
}

fn table_vertical_border(
    col_widths: &[usize],
    left: char,
    straight: char,
    intersect: char,
    right: char,
) -> Span<'static> {
    let mut out = String::with_capacity(col_widths.iter().sum::<usize>() + col_widths.len() + 1);
    out.push(left);
    for (i, &col_width) in col_widths.iter().enumerate() {
        (0..col_width).for_each(|_| out.push(straight));
        if i < col_widths.len() - 1 {
            out.push(intersect);
        } else {
            out.push(right);
        }
    }

    out.into()
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
