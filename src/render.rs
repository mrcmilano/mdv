use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

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
            Event::End(TagEnd::Paragraph) => {
                blocks.push(Block::Paragraph {
                    spans: std::mem::take(&mut current_spans),
                });
            }
            Event::End(TagEnd::Heading(level)) => {
                let level = heading_level_to_u8(level);
                let spans = std::mem::take(&mut current_spans);
                let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                headings.push(TocEntry {
                    level,
                    text,
                    block_index: blocks.len(),
                });
                blocks.push(Block::Heading { level, spans });
            }
            Event::Rule => {
                blocks.push(Block::Rule);
            }
            Event::Text(text) => {
                current_spans.push(Span {
                    text: text.into_string(),
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
        let doc = build_document("# Hello");
        assert_eq!(
            doc.blocks,
            vec![Block::Heading {
                level: 1,
                spans: vec![Span {
                    text: "Hello".to_string(),
                    style: Style::default(),
                }],
            }]
        );
        assert_eq!(
            doc.headings,
            vec![TocEntry {
                level: 1,
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
}
