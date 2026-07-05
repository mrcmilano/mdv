use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::style;
use crate::style::{Span, Style};

/// Block-level structure before wrapping. `Table`/`Alignment` are added when
/// Phase 4 lands (see plan Open questions).
// `CodeBlock`/`BlockQuote` are the build plan's own Section 4 variant names
// (not a style choice this crate is free to change).
#[allow(clippy::enum_variant_names)]
#[derive(Debug, Clone, PartialEq)]
pub enum Block {
    Heading {
        level: u8,
        spans: Vec<Span>,
    },
    Paragraph {
        spans: Vec<Span>,
    },
    Rule,
    CodeBlock {
        language: Option<String>,
        lines: Vec<String>,
    },
    BlockQuote {
        blocks: Vec<Block>,
    },
    List {
        ordered: Option<u64>,
        items: Vec<ListItem>,
    },
    Html {
        lines: Vec<String>,
    },
    FootnoteDef {
        label: String,
        blocks: Vec<Block>,
    },
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

/// Which block-level container a `parse_blocks` call is currently filling.
/// Determines which `TagEnd` stops the recursion and hands control back to
/// the caller. `TopLevel` never matches an `End` — it only stops when the
/// event iterator is exhausted.
#[derive(PartialEq)]
enum Container {
    TopLevel,
    BlockQuote,
    Item,
    FootnoteDefinition,
}

impl Container {
    fn is_closed_by(&self, end: &TagEnd) -> bool {
        matches!(
            (self, end),
            (Container::BlockQuote, TagEnd::BlockQuote(_))
                | (Container::Item, TagEnd::Item)
                | (Container::FootnoteDefinition, TagEnd::FootnoteDefinition)
        )
    }
}

/// Mutable parse state threaded through the recursive-descent block parser.
/// Inline-content accumulation (`style_stack`/`current_spans`/`link_stack`/
/// `image_alt_stack`) is a single shared buffer rather than one per nesting
/// level: content only ever accumulates depth-first, and is always flushed
/// into a `Block::Paragraph` (see `flush_pending_paragraph`) before any
/// recursive descent into a nested block-level construct, so nothing is ever
/// lost or double-buffered across recursion.
struct ParseCtx {
    style_stack: Vec<Style>,
    current_spans: Vec<Span>,
    link_stack: Vec<(String, LinkTextStart)>,
    image_alt_stack: Vec<String>,
    headings: Vec<TocEntry>,
    /// Footnote definitions collected in encounter order regardless of their
    /// source position — Section 5 requires them rendered at the document's
    /// end under a rule, not where they're physically defined.
    footnotes: Vec<(String, Vec<Block>)>,
}

impl ParseCtx {
    fn new() -> Self {
        ParseCtx {
            style_stack: vec![Style::default()],
            current_spans: Vec::new(),
            link_stack: Vec::new(),
            image_alt_stack: Vec::new(),
            headings: Vec::new(),
            footnotes: Vec::new(),
        }
    }
}

/// Flushes any inline content accumulated directly at block scope (a tight
/// list item's content arrives as bare inline events with no `Paragraph`
/// wrapper — see the Phase 3 plan's event-stream investigation) into a
/// trailing `Block::Paragraph`. A no-op when nothing is pending, which is
/// always the case for content that *was* wrapped in an explicit `Paragraph`.
fn flush_pending_paragraph(blocks: &mut Vec<Block>, ctx: &mut ParseCtx) {
    if !ctx.current_spans.is_empty() {
        blocks.push(Block::Paragraph {
            spans: std::mem::take(&mut ctx.current_spans),
        });
    }
}

/// Consumes a whole unhandled Start/End subtree (e.g. `Table`, not
/// implemented until Phase 4) so the event stream stays balanced. Called
/// right after the subtree's `Start` event has already been consumed.
fn skip_subtree<'a>(events: &mut impl Iterator<Item = Event<'a>>) {
    let mut depth: u32 = 1;
    for event in events.by_ref() {
        match event {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;
                if depth == 0 {
                    return;
                }
            }
            _ => {}
        }
    }
}

/// Splits `text` on `\n`, dropping the single trailing empty element that a
/// source string ending in `\n` always produces. Shared by `CodeBlock` and
/// `HtmlBlock` content, whose source lines both arrive as one or more `Text`/
/// `Html` chunks each already carrying their own trailing newline.
fn split_source_lines(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = text.split('\n').map(str::to_string).collect();
    if lines.last().is_some_and(String::is_empty) {
        lines.pop();
    }
    lines
}

/// Forces `dim = true` onto every `Span` reachable inside `blocks`, including
/// through nested `BlockQuote`/`List`/`FootnoteDef` structure (Section 5:
/// "quoted text Dim", applied recursively — see plan Open questions).
/// `CodeBlock`/`Html` content has no per-span styling of its own (their
/// color/gutter is applied uniformly by `layout.rs`), so there is nothing to
/// dim on those variants.
fn force_dim(blocks: &mut [Block]) {
    for block in blocks {
        match block {
            Block::Heading { spans, .. } | Block::Paragraph { spans } => {
                for span in spans {
                    span.style.dim = true;
                }
            }
            Block::BlockQuote { blocks } | Block::FootnoteDef { blocks, .. } => {
                force_dim(blocks);
            }
            Block::List { items, .. } => {
                for item in items {
                    force_dim(&mut item.blocks);
                }
            }
            Block::CodeBlock { .. } | Block::Html { .. } | Block::Rule => {}
        }
    }
}

/// Parses a `Block::List`'s items: consumes `Item`s (each recursively parsed
/// via `parse_blocks`) until the list's own `End(List)`.
///
/// `checked` is read via `.peek()` into a variable local to *this* loop
/// iteration, not a shared `ParseCtx` field: an earlier version stashed it in
/// `ParseCtx` between seeing `TaskListMarker` and consuming it after
/// `parse_blocks` returned, but when an outer task item's own content
/// contains a nested list, that nested list's `parse_list` call runs (and
/// reads/clears the field) *before* the outer call gets to consume it —
/// stealing the outer item's checked state and leaking it onto an unrelated
/// inner item. Peeking here, before recursing into the item's content at
/// all, keeps the value properly scoped per item.
fn parse_list<'a, I: Iterator<Item = Event<'a>>>(
    events: &mut std::iter::Peekable<I>,
    ctx: &mut ParseCtx,
    depth: u32,
) -> Vec<ListItem> {
    let mut items = Vec::new();
    while let Some(event) = events.next() {
        match event {
            Event::End(TagEnd::List(_)) => break,
            Event::Start(Tag::Item) => {
                let checked = match events.peek() {
                    Some(Event::TaskListMarker(_)) => {
                        let Some(Event::TaskListMarker(checked)) = events.next() else {
                            unreachable!()
                        };
                        Some(checked)
                    }
                    _ => None,
                };
                let blocks = parse_blocks(events, ctx, depth, Container::Item);
                items.push(ListItem { checked, blocks });
            }
            _ => {}
        }
    }
    items
}

/// Recursive-descent block parser. Handles one block-level container's worth
/// of content — the whole document (`Container::TopLevel`, stopping only when
/// `events` is exhausted) or one nested container (stopping at the `TagEnd`
/// `container` recognizes, per `Container::is_closed_by`). `depth` is the
/// list/blockquote nesting depth, used only to decide whether a `Heading`
/// contributes a `TocEntry` (see the Phase 3 plan's Open questions: TOC scope
/// is top-level-only, since `TocEntry::block_index` only makes sense against
/// top-level `Document.blocks`).
///
/// Inline-level events (`Text`, `Code`, breaks, styles, links, images,
/// footnote references, inline HTML) are handled exactly as before Phase 3,
/// accumulating into `ctx.current_spans` — this works unchanged whether
/// they're wrapped in an explicit `Paragraph`/`Heading` or arrive bare at
/// block scope (a tight list item's content).
fn parse_blocks<'a, I: Iterator<Item = Event<'a>>>(
    events: &mut std::iter::Peekable<I>,
    ctx: &mut ParseCtx,
    depth: u32,
    container: Container,
) -> Vec<Block> {
    let mut blocks = Vec::new();

    while let Some(event) = events.next() {
        if let Event::End(ref end) = event {
            if container.is_closed_by(end) {
                flush_pending_paragraph(&mut blocks, ctx);
                break;
            }
        }

        match event {
            Event::Start(Tag::Paragraph) => {
                ctx.current_spans.clear();
            }
            Event::Start(Tag::Heading { .. }) => {
                flush_pending_paragraph(&mut blocks, ctx);
                ctx.current_spans.clear();
            }
            Event::Start(Tag::Strong) => {
                let mut style = *ctx.style_stack.last().unwrap();
                style.bold = true;
                ctx.style_stack.push(style);
            }
            Event::Start(Tag::Emphasis) => {
                let mut style = *ctx.style_stack.last().unwrap();
                style.italic = true;
                ctx.style_stack.push(style);
            }
            Event::Start(Tag::Strikethrough) => {
                let mut style = *ctx.style_stack.last().unwrap();
                style.strikethrough = true;
                ctx.style_stack.push(style);
            }
            Event::Start(Tag::Link { dest_url, .. }) => {
                let start = match ctx.image_alt_stack.last() {
                    Some(alt) => LinkTextStart::Alt(alt.len()),
                    None => LinkTextStart::Spans(ctx.current_spans.len()),
                };
                ctx.link_stack.push((dest_url.into_string(), start));
                let mut style = *ctx.style_stack.last().unwrap();
                style.fg = Some(style::LINK);
                style.underline = true;
                ctx.style_stack.push(style);
            }
            Event::Start(Tag::Image { .. }) => {
                ctx.image_alt_stack.push(String::new());
            }
            Event::End(TagEnd::Paragraph) => {
                blocks.push(Block::Paragraph {
                    spans: std::mem::take(&mut ctx.current_spans),
                });
            }
            Event::End(TagEnd::Heading(level)) => {
                let level = heading_level_to_u8(level);
                let mut spans = std::mem::take(&mut ctx.current_spans);
                if level == 1 {
                    for span in &mut spans {
                        span.text = span.text.to_uppercase();
                    }
                }
                if depth == 0 {
                    let text: String = spans.iter().map(|s| s.text.as_str()).collect();
                    ctx.headings.push(TocEntry {
                        level,
                        text,
                        block_index: blocks.len(),
                    });
                }
                blocks.push(Block::Heading { level, spans });
            }
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough) => {
                ctx.style_stack.pop();
            }
            Event::End(TagEnd::Link) => {
                ctx.style_stack.pop();
                let (dest_url, start) = ctx.link_stack.pop().unwrap();
                let rendered_text = match start {
                    LinkTextStart::Spans(i) => ctx.current_spans[i..]
                        .iter()
                        .map(|s| s.text.as_str())
                        .collect(),
                    // `image_alt_stack`'s top is guaranteed to still be the
                    // same entry the Link started under: Image Start/End
                    // pairs are always fully balanced within a Link's own
                    // Start/End, so the stack depth here matches Start(Link)'s.
                    LinkTextStart::Alt(i) => ctx
                        .image_alt_stack
                        .last()
                        .map(|alt| alt[i..].to_string())
                        .unwrap_or_default(),
                };
                // Section 5's own operational definition of "autolink": the
                // link's rendered plain text equals its destination URL.
                if rendered_text != dest_url {
                    let suffix = format!(" ({dest_url})");
                    match ctx.image_alt_stack.last_mut() {
                        // Alt text is always plain/unstyled, so the suffix
                        // just extends it rather than becoming its own Span.
                        Some(alt) => alt.push_str(&suffix),
                        None => ctx.current_spans.push(Span {
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
                let alt = ctx.image_alt_stack.pop().unwrap_or_default();
                match ctx.image_alt_stack.last_mut() {
                    // Nested inside another Image's alt text: fold this
                    // image down to its plain alt text (no brackets — the
                    // outer alt text is plain, unstyled content) rather than
                    // leaking a Span into current_spans while the outer
                    // image is still open.
                    Some(outer_alt) => outer_alt.push_str(&alt),
                    None => ctx.current_spans.push(Span {
                        text: format!("[image: {alt}]"),
                        style: Style {
                            dim: true,
                            ..Style::default()
                        },
                    }),
                }
            }
            Event::Rule => {
                flush_pending_paragraph(&mut blocks, ctx);
                blocks.push(Block::Rule);
            }
            Event::Start(Tag::CodeBlock(kind)) => {
                flush_pending_paragraph(&mut blocks, ctx);
                let language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) if !lang.is_empty() => {
                        Some(lang.into_string())
                    }
                    _ => None,
                };
                let mut text = String::new();
                for event in events.by_ref() {
                    match event {
                        Event::Text(t) => text.push_str(&t),
                        Event::End(TagEnd::CodeBlock) => break,
                        _ => break,
                    }
                }
                blocks.push(Block::CodeBlock {
                    language,
                    lines: split_source_lines(&text),
                });
            }
            Event::Start(Tag::BlockQuote(_)) => {
                flush_pending_paragraph(&mut blocks, ctx);
                let mut inner = parse_blocks(events, ctx, depth + 1, Container::BlockQuote);
                force_dim(&mut inner);
                blocks.push(Block::BlockQuote { blocks: inner });
            }
            Event::Start(Tag::List(start)) => {
                flush_pending_paragraph(&mut blocks, ctx);
                let items = parse_list(events, ctx, depth + 1);
                blocks.push(Block::List {
                    ordered: start,
                    items,
                });
            }
            Event::Start(Tag::HtmlBlock) => {
                flush_pending_paragraph(&mut blocks, ctx);
                let mut text = String::new();
                for event in events.by_ref() {
                    match event {
                        Event::Html(t) => text.push_str(&t),
                        Event::End(TagEnd::HtmlBlock) => break,
                        _ => break,
                    }
                }
                blocks.push(Block::Html {
                    lines: split_source_lines(&text),
                });
            }
            Event::Start(Tag::FootnoteDefinition(label)) => {
                flush_pending_paragraph(&mut blocks, ctx);
                let inner = parse_blocks(events, ctx, depth + 1, Container::FootnoteDefinition);
                ctx.footnotes.push((label.into_string(), inner));
            }
            Event::InlineHtml(text) => {
                if let Some(alt) = ctx.image_alt_stack.last_mut() {
                    alt.push_str(&text);
                } else {
                    ctx.current_spans.push(Span {
                        text: text.into_string(),
                        style: Style {
                            dim: true,
                            ..*ctx.style_stack.last().unwrap()
                        },
                    });
                }
            }
            Event::FootnoteReference(label) => {
                let span = Span {
                    text: format!("[^{label}]"),
                    style: Style {
                        dim: true,
                        ..*ctx.style_stack.last().unwrap()
                    },
                };
                match ctx.image_alt_stack.last_mut() {
                    Some(alt) => alt.push_str(&span.text),
                    None => ctx.current_spans.push(span),
                }
            }
            Event::Text(text) => {
                if let Some(alt) = ctx.image_alt_stack.last_mut() {
                    alt.push_str(&text);
                } else {
                    ctx.current_spans.push(Span {
                        text: text.into_string(),
                        style: *ctx.style_stack.last().unwrap(),
                    });
                }
            }
            Event::Code(text) => {
                if let Some(alt) = ctx.image_alt_stack.last_mut() {
                    alt.push_str(&text);
                } else {
                    let mut code_style = *ctx.style_stack.last().unwrap();
                    code_style.fg = Some(style::CODE);
                    ctx.current_spans.push(Span {
                        text: text.into_string(),
                        style: code_style,
                    });
                }
            }
            Event::SoftBreak => {
                if let Some(alt) = ctx.image_alt_stack.last_mut() {
                    alt.push(' ');
                } else {
                    ctx.current_spans.push(Span {
                        text: " ".to_string(),
                        style: *ctx.style_stack.last().unwrap(),
                    });
                }
            }
            Event::HardBreak => {
                // Sentinel recognized by layout.rs as a forced line break.
                // Safe because pulldown-cmark never places a literal '\n'
                // inside a Text event (source line breaks arrive as separate
                // SoftBreak/HardBreak events), so this can only mean "break
                // here" and never appear as real content.
                ctx.current_spans.push(Span {
                    text: "\n".to_string(),
                    style: Style::default(),
                });
            }
            Event::Start(_) => {
                skip_subtree(events);
            }
            _ => {}
        }
    }

    blocks
}

/// Parses `markdown` into a `Document`. Block-level tags this phase doesn't
/// implement yet (`Table` and friends — Phase 4) are skipped as a balanced
/// subtree via `skip_subtree`, contributing no `Block`/`Span` and never
/// panicking.
pub fn build_document(markdown: &str) -> Document {
    let options = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_FOOTNOTES;
    let mut events = Parser::new_ext(markdown, options).peekable();

    let mut ctx = ParseCtx::new();
    let mut blocks = parse_blocks(&mut events, &mut ctx, 0, Container::TopLevel);

    if !ctx.footnotes.is_empty() {
        blocks.push(Block::Rule);
        for (label, footnote_blocks) in ctx.footnotes.drain(..) {
            blocks.push(Block::FootnoteDef {
                label,
                blocks: footnote_blocks,
            });
        }
    }

    Document {
        blocks,
        headings: ctx.headings,
    }
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

    #[test]
    fn fenced_code_block_with_language_captures_lines_and_language() {
        let doc = build_document("```rust\nfn main() {}\nlet x = 1;\n```");
        assert_eq!(
            doc.blocks,
            vec![Block::CodeBlock {
                language: Some("rust".to_string()),
                lines: vec!["fn main() {}".to_string(), "let x = 1;".to_string()],
            }]
        );
    }

    #[test]
    fn fenced_code_block_without_language_has_no_language() {
        let doc = build_document("```\nplain\n```");
        assert_eq!(
            doc.blocks,
            vec![Block::CodeBlock {
                language: None,
                lines: vec!["plain".to_string()],
            }]
        );
    }

    #[test]
    fn indented_code_block_has_no_language() {
        let doc = build_document("    indented line");
        assert_eq!(
            doc.blocks,
            vec![Block::CodeBlock {
                language: None,
                lines: vec!["indented line".to_string()],
            }]
        );
    }

    #[test]
    fn empty_fenced_code_block_has_no_lines() {
        let doc = build_document("```\n```");
        assert_eq!(
            doc.blocks,
            vec![Block::CodeBlock {
                language: None,
                lines: vec![],
            }]
        );
    }

    #[test]
    fn single_level_blockquote_dims_its_paragraph() {
        let doc = build_document("> quoted text");
        assert_eq!(
            doc.blocks,
            vec![Block::BlockQuote {
                blocks: vec![Block::Paragraph {
                    spans: vec![Span {
                        text: "quoted text".to_string(),
                        style: Style {
                            dim: true,
                            ..Style::default()
                        },
                    }],
                }],
            }]
        );
    }

    #[test]
    fn nested_blockquote_dims_both_levels() {
        let doc = build_document("> outer\n> > inner");
        assert_eq!(
            doc.blocks,
            vec![Block::BlockQuote {
                blocks: vec![
                    Block::Paragraph {
                        spans: vec![Span {
                            text: "outer".to_string(),
                            style: Style {
                                dim: true,
                                ..Style::default()
                            },
                        }],
                    },
                    Block::BlockQuote {
                        blocks: vec![Block::Paragraph {
                            spans: vec![Span {
                                text: "inner".to_string(),
                                style: Style {
                                    dim: true,
                                    ..Style::default()
                                },
                            }],
                        }],
                    },
                ],
            }]
        );
    }

    #[test]
    fn blockquote_containing_code_block_leaves_code_lines_undimmed() {
        // force_dim only touches Span-bearing blocks (Heading/Paragraph); a
        // CodeBlock's lines carry no per-span style (layout.rs colors them
        // uniformly), so there's nothing to dim.
        let doc = build_document("> ```\n> code\n> ```");
        assert_eq!(
            doc.blocks,
            vec![Block::BlockQuote {
                blocks: vec![Block::CodeBlock {
                    language: None,
                    lines: vec!["code".to_string()],
                }],
            }]
        );
    }

    #[test]
    fn unordered_list_produces_bullet_items() {
        let doc = build_document("- a\n- b");
        assert_eq!(
            doc.blocks,
            vec![Block::List {
                ordered: None,
                items: vec![
                    ListItem {
                        checked: None,
                        blocks: vec![Block::Paragraph {
                            spans: vec![Span {
                                text: "a".to_string(),
                                style: Style::default(),
                            }],
                        }],
                    },
                    ListItem {
                        checked: None,
                        blocks: vec![Block::Paragraph {
                            spans: vec![Span {
                                text: "b".to_string(),
                                style: Style::default(),
                            }],
                        }],
                    },
                ],
            }]
        );
    }

    #[test]
    fn ordered_list_keeps_custom_start_number() {
        let doc = build_document("3. a\n4. b");
        match &doc.blocks[0] {
            Block::List { ordered, items } => {
                assert_eq!(*ordered, Some(3));
                assert_eq!(items.len(), 2);
            }
            other => panic!("expected List, got {other:?}"),
        }
    }

    #[test]
    fn task_list_items_carry_checked_state() {
        let doc = build_document("- [ ] todo\n- [x] done");
        match &doc.blocks[0] {
            Block::List { items, .. } => {
                assert_eq!(items[0].checked, Some(false));
                assert_eq!(items[1].checked, Some(true));
            }
            other => panic!("expected List, got {other:?}"),
        }
    }

    #[test]
    fn checked_task_item_containing_a_nested_list_keeps_its_own_checked_state() {
        // Adversarial-review finding: a shared `ParseCtx` scratch field for
        // the pending TaskListMarker was read-and-cleared by the *inner*
        // list's own item before the outer task item consumed it, stealing
        // the outer's checked state and leaking it onto the unrelated inner
        // item. The fix scopes it to a `parse_list` loop-local via peek.
        let doc = build_document("- [x] outer\n  - inner");
        let Block::List { items: outer, .. } = &doc.blocks[0] else {
            panic!("expected top-level List");
        };
        assert_eq!(outer[0].checked, Some(true));
        let Block::List { items: inner, .. } = &outer[0].blocks[1] else {
            panic!(
                "expected nested List as outer item's second block, got {:?}",
                outer[0].blocks
            );
        };
        assert_eq!(inner[0].checked, None);
    }

    #[test]
    fn three_level_nested_mixed_list() {
        let doc = build_document("- a\n  1. b\n     - c\n- d");
        let Block::List { items, .. } = &doc.blocks[0] else {
            panic!("expected top-level List");
        };
        assert_eq!(items.len(), 2);
        // First item: "a" paragraph plus a nested ordered list.
        assert_eq!(items[0].blocks.len(), 2);
        let Block::List {
            ordered: Some(1),
            items: level2,
        } = &items[0].blocks[1]
        else {
            panic!("expected nested ordered List, got {:?}", items[0].blocks[1]);
        };
        assert_eq!(level2.len(), 1);
        assert_eq!(level2[0].blocks.len(), 2);
        assert!(matches!(
            level2[0].blocks[1],
            Block::List { ordered: None, .. }
        ));
    }

    #[test]
    fn html_block_captures_verbatim_lines() {
        let doc = build_document("<div>\nhello\n</div>\n\npara after");
        assert_eq!(
            doc.blocks,
            vec![
                Block::Html {
                    lines: vec![
                        "<div>".to_string(),
                        "hello".to_string(),
                        "</div>".to_string()
                    ],
                },
                Block::Paragraph {
                    spans: vec![Span {
                        text: "para after".to_string(),
                        style: Style::default(),
                    }],
                },
            ]
        );
    }

    #[test]
    fn inline_html_becomes_a_dim_span_inside_the_paragraph() {
        let spans = only_paragraph_spans("before <span>x</span> after");
        assert_eq!(
            spans,
            vec![
                Span {
                    text: "before ".to_string(),
                    style: Style::default(),
                },
                Span {
                    text: "<span>".to_string(),
                    style: Style {
                        dim: true,
                        ..Style::default()
                    },
                },
                Span {
                    text: "x".to_string(),
                    style: Style::default(),
                },
                Span {
                    text: "</span>".to_string(),
                    style: Style {
                        dim: true,
                        ..Style::default()
                    },
                },
                Span {
                    text: " after".to_string(),
                    style: Style::default(),
                },
            ]
        );
    }

    #[test]
    fn footnote_reference_and_definition_round_trip() {
        let doc = build_document("text[^1] more\n\n[^1]: note body");
        assert_eq!(
            doc.blocks,
            vec![
                Block::Paragraph {
                    spans: vec![
                        Span {
                            text: "text".to_string(),
                            style: Style::default(),
                        },
                        Span {
                            text: "[^1]".to_string(),
                            style: Style {
                                dim: true,
                                ..Style::default()
                            },
                        },
                        Span {
                            text: " more".to_string(),
                            style: Style::default(),
                        },
                    ],
                },
                Block::Rule,
                Block::FootnoteDef {
                    label: "1".to_string(),
                    blocks: vec![Block::Paragraph {
                        spans: vec![Span {
                            text: "note body".to_string(),
                            style: Style::default(),
                        }],
                    }],
                },
            ]
        );
    }

    #[test]
    fn footnote_definition_before_reference_in_source_still_renders_at_the_end() {
        let doc = build_document("[^1]: note body\n\ntext[^1] more");
        assert_eq!(doc.blocks.len(), 3);
        assert!(matches!(doc.blocks[0], Block::Paragraph { .. }));
        assert_eq!(doc.blocks[1], Block::Rule);
        assert!(matches!(doc.blocks[2], Block::FootnoteDef { .. }));
    }
}
