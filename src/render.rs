use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::style;
use crate::style::{Span, Style};

/// Block-level structure before wrapping. `Table`/`Alignment` are added when
/// Phase 4 lands (see plan Open questions).
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading { level: u8, spans: Vec<Span> },
    Paragraph { spans: Vec<Span> },
    Rule,
    CodeBlock { language: Option<String>, lines: Vec<String> },
    BlockQuote { blocks: Vec<Block> },
    List { ordered: Option<u64>, items: Vec<ListItem> },
    Html { lines: Vec<String> },
    FootnoteDef { label: String, blocks: Vec<Block> },
}

/// One item of a `Block::List`. `checked` is `None` for a non-task item,
/// `Some(bool)` for a task-list item (Phase 3 plan Open questions: Section
/// 4's illustrative `items: Vec<Vec<Block>>` has no field for this state).
#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub checked: Option<bool>,
    pub blocks: Vec<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TocEntry {
    pub level: u8,
    pub text: String,
    pub block_index: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    pub blocks: Vec<Block>,
    pub headings: Vec<TocEntry>,
}

fn heading_level_to_u8(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 => 1,
        HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// Where a currently-open `Link`'s rendered content is accumulating: the
/// common case is `current_spans`, but a `Link` nested inside an `Image`'s
/// alt text accumulates in that image's entry in `image_alt_stack` instead
/// (alt text is always plain, unstyled text — see `image_alt_stack`).
enum LinkTextStart {
    Spans(usize),
    Alt(usize),
}

/// Parses `markdown` into a `Document`. Any event this phase doesn't handle
/// (block-level tags `Table`/`List`/`BlockQuote`/`CodeBlock`/
/// `FootnoteDefinition`/`HtmlBlock`, and leaf events `Html`/`InlineHtml`/
/// `TaskListMarker`/`FootnoteReference`) is silently consumed so the flat
/// iteration stays balanced — it contributes no `Block`/`Span` and never
/// panics. Temporary until Phase 3/4 add real handling.
pub fn build_document(markdown: &str) -> Document {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let parser = Parser::new_ext(markdown, options);

    let mut blocks = Vec::new();
    let mut headings = Vec::new();
    let mut current_spans: Vec<Span> = Vec::new();
    // Nestable style stack (task 3): the top is the style applied to the next
    // Text/Code event. Starts with one default entry so `.last()` is always
    // valid; Strong/Emphasis/Strikethrough/Link push a modified copy and pop
    // it on their matching End.
    let mut style_stack: Vec<Style> = vec![Style::default()];
    // One entry per currently-open Link: its destination URL, and where its
    // content started, so End(Link) can recompute the link's rendered plain
    // text for the autolink comparison. Content lands in `current_spans`
    // normally, but if the link is itself nested inside an Image's alt text
    // (unusual but valid Markdown pulldown-cmark does emit), its content
    // instead accumulates in `image_alt_stack`'s top entry — `LinkTextStart`
    // records which buffer to look at.
    let mut link_stack: Vec<(String, LinkTextStart)> = Vec::new();
    // Stack of in-progress alt-text accumulators, one per currently-open
    // Image (innermost last): nested markup is flattened to plain text and
    // accumulated here rather than pushed as styled spans (Section 5: alt
    // text only, never fetch/render). A stack, not a single slot, because an
    // Image can itself be nested inside another Image's alt text.
    let mut image_alt_stack: Vec<String> = Vec::new();
    // Depth counter for an unhandled Start/End tag subtree currently being
    // skipped; >0 means every event is consumed until it unwinds to 0.
    let mut skip_depth: u32 = 0;

    for event in parser {
        if skip_depth > 0 {
            match event {
                Event::Start(_) => skip_depth += 1,
                Event::End(_) => skip_depth -= 1,
                _ => {}
            }
            continue;
        }

        match event {
            Event::Start(Tag::Paragraph) => {
                current_spans.clear();
            }
            Event::Start(Tag::Heading { .. }) => {
                current_spans.clear();
            }
            Event::Start(Tag::Strong) => {
                let mut style = *style_stack.last().unwrap();
                style.bold = true;
                style_stack.push(style);
            }
            Event::Start(Tag::Emphasis) => {
                let mut style = *style_stack.last().unwrap();
                style.italic = true;
                style_stack.push(style);
            }
            Event::Start(Tag::Strikethrough) => {
                let mut style = *style_stack.last().unwrap();
                style.strikethrough = true;
                style_stack.push(style);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let start = match image_alt_stack.last() {
                    Some(alt) => LinkTextStart::Alt(alt.len()),
                    None => LinkTextStart::Spans(current_spans.len()),
                };
                link_stack.push((dest_url.into_string(), start));
                let mut style = *style_stack.last().unwrap();
                style.fg = Some(style::LINK);
                style.underline = true;
                style_stack.push(style);
            }
            Event::Start(Tag::Image { .. }) => {
                image_alt_stack.push(String::new());
            }
            Event::End(TagEnd::Paragraph) => {
                blocks.push(Block::Paragraph {
                    spans: std::mem::take(&mut current_spans),
                });
            }
            Event::End(TagEnd::Heading(level)) => {
                let level = heading_level_to_u8(level);
                let mut spans = std::mem::take(&mut current_spans);
                if level == 1 {
                    for span in &mut spans {
                        span.text = span.text.to_uppercase();
                    }
                }
                let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                headings.push(TocEntry {
                    level,
                    text,
                    block_index: blocks.len(),
                });
                blocks.push(Block::Heading { level, spans });
            }
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough) => {
                style_stack.pop();
            }
            Event::End(TagEnd::Link) => {
                style_stack.pop();
                let (dest_url, start) = link_stack.pop().unwrap();
                let rendered_text = match start {
                    LinkTextStart::Spans(i) => {
                        current_spans[i..].iter().map(|s| s.text.as_str()).collect()
                    }
                    // `image_alt_stack`'s top is guaranteed to still be the
                    // same entry the Link started under: Image Start/End
                    // pairs are always fully balanced within a Link's own
                    // Start/End, so the stack depth here matches Start(Link)'s.
                    LinkTextStart::Alt(i) => image_alt_stack
                        .last()
                        .map(|alt| alt[i..].to_string())
                        .unwrap_or_default(),
                };
                // Section 5's own operational definition of "autolink": the
                // link's rendered plain text equals its destination URL.
                if rendered_text != dest_url {
                    let suffix = format!(" ({dest_url})");
                    match image_alt_stack.last_mut() {
                        // Alt text is always plain/unstyled, so the suffix
                        // just extends it rather than becoming its own Span.
                        Some(alt) => alt.push_str(&suffix),
                        None => current_spans.push(Span {
                            text: suffix,
                            style: Style {
                                dim: true,
                                ..Style::default()
                            },
                        }),
                    }
                }
            }
            Event::End(TagEnd::Image) => {
                let alt = image_alt_stack.pop().unwrap_or_default();
                match image_alt_stack.last_mut() {
                    // Nested inside another Image's alt text: fold this
                    // image down to its plain alt text (no brackets — the
                    // outer alt text is plain, unstyled content) rather than
                    // leaking a Span into current_spans while the outer
                    // image is still open.
                    Some(outer_alt) => outer_alt.push_str(&alt),
                    None => current_spans.push(Span {
                        text: format!("[image: {alt}]"),
                        style: Style {
                            dim: true,
                            ..Style::default()
                        },
                    }),
                }
            }
            Event::Rule => {
                blocks.push(Block::Rule);
            }
            Event::Text(text) => {
                if let Some(alt) = image_alt_stack.last_mut() {
                    alt.push_str(&text);
                } else {
                    current_spans.push(Span {
                        text: text.into_string(),
                        style: *style_stack.last().unwrap(),
                    });
                }
            }
            Event::Code(text) => {
                if let Some(alt) = image_alt_stack.last_mut() {
                    alt.push_str(&text);
                } else {
                    let mut code_style = *style_stack.last().unwrap();
                    code_style.fg = Some(style::CODE);
                    current_spans.push(Span {
                        text: text.into_string(),
                        style: code_style,
                    });
                }
            }
            Event::SoftBreak => {
                if let Some(alt) = image_alt_stack.last_mut() {
                    alt.push(' ');
                } else {
                    current_spans.push(Span {
                        text: " ".to_string(),
                        style: *style_stack.last().unwrap(),
                    });
                }
            }
            Event::HardBreak => {
                // Sentinel recognized by layout.rs as a forced line break.
                // Safe because pulldown-cmark never places a literal '\n'
                // inside a Text event (source line breaks arrive as separate
                // SoftBreak/HardBreak events), so this can only mean "break
                // here" and never appear as real content.
                current_spans.push(Span {
                    text: "\n".to_string(),
                    style: Style::default(),
                });
            }
            Event::Start(_) => {
                skip_depth = 1;
            }
            _ => {}
        }
    }

    Document { blocks, headings }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heading_produces_heading_block_and_toc_entry() {
        // Level-2+ headings aren't uppercased (that's H1-only, see the
        // dedicated h1_uppercases_all_inline_content_including_code test).
        let doc = build_document("## Hello");
        assert_eq!(
            doc.blocks,
            vec![Block::Heading {
                level: 2,
                spans: vec![Span {
                    text: "Hello".to_string(),
                    style: Style::default(),
                }],
            }]
        );
        assert_eq!(
            doc.headings,
            vec![TocEntry {
                level: 2,
                text: "Hello".to_string(),
                block_index: 0,
            }]
        );
    }

    #[test]
    fn paragraph_produces_paragraph_block() {
        let doc = build_document("hello world");
        assert_eq!(
            doc.blocks,
            vec![Block::Paragraph {
                spans: vec![Span {
                    text: "hello world".to_string(),
                    style: Style::default(),
                }],
            }]
        );
        assert!(doc.headings.is_empty());
    }

    #[test]
    fn rule_produces_rule_block() {
        let doc = build_document("---");
        assert_eq!(doc.blocks, vec![Block::Rule]);
    }

    fn only_paragraph_spans(markdown: &str) -> Vec<Span> {
        let doc = build_document(markdown);
        match doc.blocks.into_iter().next() {
            Some(Block::Paragraph { spans }) => spans,
            other => panic!("expected a single Paragraph block, got {other:?}"),
        }
    }

    #[test]
    fn strong_sets_bold() {
        let spans = only_paragraph_spans("**bold**");
        assert_eq!(
            spans,
            vec![Span {
                text: "bold".to_string(),
                style: Style {
                    bold: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn emphasis_sets_italic() {
        let spans = only_paragraph_spans("*italic*");
        assert_eq!(
            spans,
            vec![Span {
                text: "italic".to_string(),
                style: Style {
                    italic: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn strikethrough_sets_strikethrough() {
        let spans = only_paragraph_spans("~~struck~~");
        assert_eq!(
            spans,
            vec![Span {
                text: "struck".to_string(),
                style: Style {
                    strikethrough: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn inline_code_gets_code_color() {
        let spans = only_paragraph_spans("`code`");
        assert_eq!(
            spans,
            vec![Span {
                text: "code".to_string(),
                style: Style {
                    fg: Some(style::CODE),
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn nested_bold_and_italic_combine() {
        let spans = only_paragraph_spans("**bold *italic* text**");
        assert_eq!(
            spans,
            vec![
                Span {
                    text: "bold ".to_string(),
                    style: Style {
                        bold: true,
                        ..Style::default()
                    },
                },
                Span {
                    text: "italic".to_string(),
                    style: Style {
                        bold: true,
                        italic: true,
                        ..Style::default()
                    },
                },
                Span {
                    text: " text".to_string(),
                    style: Style {
                        bold: true,
                        ..Style::default()
                    },
                },
            ]
        );
    }

    #[test]
    fn link_with_distinct_url_gets_dim_url_suffix() {
        let spans = only_paragraph_spans("[text](https://example.com)");
        assert_eq!(
            spans,
            vec![
                Span {
                    text: "text".to_string(),
                    style: Style {
                        fg: Some(style::LINK),
                        underline: true,
                        ..Style::default()
                    },
                },
                Span {
                    text: " (https://example.com)".to_string(),
                    style: Style {
                        dim: true,
                        ..Style::default()
                    },
                },
            ]
        );
    }

    #[test]
    fn autolink_skips_the_url_suffix() {
        let spans = only_paragraph_spans("<https://example.com>");
        assert_eq!(
            spans,
            vec![Span {
                text: "https://example.com".to_string(),
                style: Style {
                    fg: Some(style::LINK),
                    underline: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn image_becomes_dim_alt_placeholder() {
        let spans = only_paragraph_spans("![a cat](cat.png)");
        assert_eq!(
            spans,
            vec![Span {
                text: "[image: a cat]".to_string(),
                style: Style {
                    dim: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn hard_break_produces_newline_sentinel() {
        let spans = only_paragraph_spans("line one  \nline two");
        assert_eq!(
            spans,
            vec![
                Span {
                    text: "line one".to_string(),
                    style: Style::default(),
                },
                Span {
                    text: "\n".to_string(),
                    style: Style::default(),
                },
                Span {
                    text: "line two".to_string(),
                    style: Style::default(),
                },
            ]
        );
    }

    #[test]
    fn soft_break_produces_a_space() {
        let spans = only_paragraph_spans("line one\nline two");
        assert_eq!(
            spans,
            vec![
                Span {
                    text: "line one".to_string(),
                    style: Style::default(),
                },
                Span {
                    text: " ".to_string(),
                    style: Style::default(),
                },
                Span {
                    text: "line two".to_string(),
                    style: Style::default(),
                },
            ]
        );
    }

    #[test]
    fn link_nested_inside_image_alt_folds_into_a_single_placeholder() {
        // Adversarial-review finding (ref #4, task 10): a Link nested inside
        // an Image's alt text must not leak its url-suffix span into the
        // paragraph — it must fold into the one dim image placeholder.
        let spans = only_paragraph_spans("![a [link](url) cat](cat.png)");
        assert_eq!(
            spans,
            vec![Span {
                text: "[image: a link (url) cat]".to_string(),
                style: Style {
                    dim: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn autolink_nested_inside_image_alt_skips_the_url_suffix() {
        let spans = only_paragraph_spans("![see <https://example.com>](cat.png)");
        assert_eq!(
            spans,
            vec![Span {
                text: "[image: see https://example.com]".to_string(),
                style: Style {
                    dim: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn image_nested_inside_image_alt_folds_into_a_single_placeholder() {
        // Adversarial-review finding (ref #4, task 10): an Image nested
        // inside another Image's alt text must not leak a stray Span (or
        // plain undimmed text) into the paragraph — it must fold into one
        // dim placeholder for the outer image.
        let spans = only_paragraph_spans("![outer ![inner](a.png) text](b.png)");
        assert_eq!(
            spans,
            vec![Span {
                text: "[image: outer inner text]".to_string(),
                style: Style {
                    dim: true,
                    ..Style::default()
                },
            }]
        );
    }

    #[test]
    fn h1_uppercases_all_inline_content_including_code() {
        let doc = build_document("# hello `code`");
        assert_eq!(
            doc.blocks,
            vec![Block::Heading {
                level: 1,
                spans: vec![
                    Span {
                        text: "HELLO ".to_string(),
                        style: Style::default(),
                    },
                    Span {
                        text: "CODE".to_string(),
                        style: Style {
                            fg: Some(style::CODE),
                            ..Style::default()
                        },
                    },
                ],
            }]
        );
    }
}
