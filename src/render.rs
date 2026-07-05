use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::style;
use crate::style::{Span, Style};

/// Block-level structure before wrapping. Only the variants Phase 2 actually
/// constructs exist here — `CodeBlock`/`BlockQuote`/`List`/`Table`/etc. are
/// added when their phases land (see plan Open questions).
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading { level: u8, spans: Vec<Span> },
    Paragraph { spans: Vec<Span> },
    Rule,
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
            Event::Rule => {
                blocks.push(Block::Rule);
            }
            Event::Text(text) => {
                current_spans.push(Span {
                    text: text.into_string(),
                    style: *style_stack.last().unwrap(),
                });
            }
            Event::Code(text) => {
                let mut code_style = *style_stack.last().unwrap();
                code_style.fg = Some(style::CODE);
                current_spans.push(Span {
                    text: text.into_string(),
                    style: code_style,
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
