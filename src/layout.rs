use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::render::{Block, Document};
use crate::style;
use crate::style::{Span, Style};

/// One visual line after wrapping. Rendering = print spans left to right.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Line {
    pub spans: Vec<Span>,
}

/// `layout::wrap`'s return shape (Open questions: a named-field struct, not a
/// tuple, for clarity at call sites).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LayoutResult {
    pub lines: Vec<Line>,
    /// First wrapped-line index per `TocEntry`, in `Document.headings` order.
    /// Populated in task 6; empty until then.
    pub heading_lines: Vec<usize>,
}

/// Security-critical sanitization (build plan Section 5), applied to every
/// span's text before wrapping: strips `\r`, replaces tabs with a single
/// space (the "4 spaces inside code blocks" branch is dead until Phase 3),
/// and replaces every other C0 control character, DEL, and C1 control with
/// U+FFFD. Neutralizes escape-sequence injection — no other ANSI/OSC
/// sequence in the source file survives to reach the terminal.
fn sanitize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\r' => {}
            '\n' => out.push('\n'),
            '\t' => out.push(' '),
            c if is_replaceable_control(c) => out.push('\u{FFFD}'),
            c => out.push(c),
        }
    }
    out
}

fn is_replaceable_control(c: char) -> bool {
    let code = c as u32;
    (0x00..=0x1F).contains(&code) || code == 0x7F || (0x80..=0x9F).contains(&code)
}

/// One piece of a word: a contiguous, space-free run of text carrying one
/// style. A word can be made of several pieces when its source spans (e.g. a
/// bold run immediately followed by an inline-code run) abut with no space
/// between them.
struct Piece {
    text: String,
    style: Style,
}

enum Token {
    Word(Vec<Piece>),
    Space,
    /// The hard-break sentinel: forces a line flush, no visible content.
    Break,
}

/// Splits a block's styled span stream into word/space/break tokens, treating
/// the spans as one continuous text stream for word-break purposes (spans
/// contribute to the same word when they abut with no space between them).
fn tokenize(spans: &[Span]) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_word: Vec<Piece> = Vec::new();

    for span in spans {
        if span.text == "\n" {
            if !current_word.is_empty() {
                tokens.push(Token::Word(std::mem::take(&mut current_word)));
            }
            tokens.push(Token::Break);
            continue;
        }

        let text = sanitize(&span.text);
        let mut buf = String::new();
        for c in text.chars() {
            if c == ' ' {
                if !buf.is_empty() {
                    current_word.push(Piece {
                        text: std::mem::take(&mut buf),
                        style: span.style,
                    });
                }
                if !current_word.is_empty() {
                    tokens.push(Token::Word(std::mem::take(&mut current_word)));
                }
                tokens.push(Token::Space);
            } else {
                buf.push(c);
            }
        }
        if !buf.is_empty() {
            current_word.push(Piece {
                text: buf,
                style: span.style,
            });
        }
    }
    if !current_word.is_empty() {
        tokens.push(Token::Word(current_word));
    }

    tokens
}

/// Finds the longest prefix of `s` (on a char boundary) whose display width
/// is `<= max_width`, returning `(prefix, rest)`. Never splits a char in
/// half; a char wider than `max_width` on its own yields an empty prefix.
fn split_at_width(s: &str, max_width: usize) -> (&str, &str) {
    let mut width = 0;
    for (i, c) in s.char_indices() {
        let w = UnicodeWidthChar::width(c).unwrap_or(0);
        if width + w > max_width {
            return (&s[..i], &s[i..]);
        }
        width += w;
    }
    (s, "")
}

/// Appends `piece` onto `lines`/`current_line`, splitting it across as many
/// lines as needed when it doesn't fit in the remaining width. Guarantees
/// forward progress even when a single character is wider than
/// `content_width` (e.g. a CJK/emoji char at width 1): such a character is
/// force-placed on its own line, allowed to overflow by one column, rather
/// than looping forever trying to fit it.
fn place_piece(
    piece: &Piece,
    content_width: usize,
    lines: &mut Vec<Line>,
    current_line: &mut Vec<Span>,
    current_width: &mut usize,
) {
    let mut remaining = piece.text.as_str();
    loop {
        if remaining.is_empty() {
            return;
        }
        let avail = content_width.saturating_sub(*current_width);
        if avail == 0 {
            flush_line(lines, current_line, current_width);
            continue;
        }
        let piece_width = UnicodeWidthStr::width(remaining);
        if piece_width <= avail {
            current_line.push(Span {
                text: remaining.to_string(),
                style: piece.style,
            });
            *current_width += piece_width;
            return;
        }
        let (fit, rest) = split_at_width(remaining, avail);
        if fit.is_empty() {
            if *current_width == 0 {
                // Even a fresh line can't fit the next char at this width.
                // Force it through so we always make progress.
                let first_len = remaining.chars().next().unwrap().len_utf8();
                let (forced, rest2) = remaining.split_at(first_len);
                current_line.push(Span {
                    text: forced.to_string(),
                    style: piece.style,
                });
                *current_width += UnicodeWidthStr::width(forced);
                remaining = rest2;
            } else {
                flush_line(lines, current_line, current_width);
            }
            continue;
        }
        current_line.push(Span {
            text: fit.to_string(),
            style: piece.style,
        });
        *current_width += UnicodeWidthStr::width(fit);
        remaining = rest;
        flush_line(lines, current_line, current_width);
    }
}

fn flush_line(lines: &mut Vec<Line>, current_line: &mut Vec<Span>, current_width: &mut usize) {
    lines.push(Line {
        spans: std::mem::take(current_line),
    });
    *current_width = 0;
}

/// Greedily word-wraps one block's span stream to `content_width`, packing
/// words onto each output `Line`. A bold (or otherwise styled) word split
/// across a wrap keeps its style on both fragments (each fragment is emitted
/// as its own `Span` carrying the original `Piece`'s style unchanged).
fn wrap_spans(spans: &[Span], content_width: usize) -> Vec<Line> {
    let tokens = tokenize(spans);
    let mut lines = Vec::new();
    let mut current_line: Vec<Span> = Vec::new();
    let mut current_width: usize = 0;
    let mut pending_spaces: usize = 0;

    for token in tokens {
        match token {
            Token::Break => {
                flush_line(&mut lines, &mut current_line, &mut current_width);
                pending_spaces = 0;
            }
            Token::Space => {
                pending_spaces += 1;
            }
            Token::Word(pieces) => {
                let word_width: usize = pieces
                    .iter()
                    .map(|p| UnicodeWidthStr::width(p.text.as_str()))
                    .sum();
                let needed = if current_width > 0 { pending_spaces } else { 0 } + word_width;
                if current_width > 0 && current_width + needed > content_width {
                    flush_line(&mut lines, &mut current_line, &mut current_width);
                    pending_spaces = 0;
                }
                if current_width > 0 && pending_spaces > 0 {
                    current_line.push(Span {
                        text: " ".repeat(pending_spaces),
                        style: Style::default(),
                    });
                    current_width += pending_spaces;
                }
                pending_spaces = 0;
                for piece in &pieces {
                    place_piece(
                        piece,
                        content_width,
                        &mut lines,
                        &mut current_line,
                        &mut current_width,
                    );
                }
            }
        }
    }
    if current_width > 0 || !current_line.is_empty() {
        lines.push(Line {
            spans: current_line,
        });
    }

    lines
}

/// Builds the spans a heading actually wraps: H1–H3 force bold and (unless a
/// span already carries its own color, e.g. inline code or a link) the
/// heading color; H4–H6 force bold only and gain a `"§ "` prefix. The
/// uppercase transform for H1 already happened in `render.rs` — this only
/// adds presentation, never touches text content (except the H4–H6 prefix).
fn heading_presentation_spans(level: u8, spans: &[Span]) -> Vec<Span> {
    let mut result = Vec::new();
    if (4..=6).contains(&level) {
        result.push(Span {
            text: "§ ".to_string(),
            style: Style {
                bold: true,
                ..Style::default()
            },
        });
    }
    for span in spans {
        let mut style = span.style;
        style.bold = true;
        if level <= 3 && style.fg.is_none() {
            style.fg = Some(style::HEADING);
        }
        result.push(Span {
            text: span.text.clone(),
            style,
        });
    }
    result
}

/// H1/H2 underline width: the heading's pre-wrap display width (its spans'
/// sanitized text concatenated), capped at `content_width` (Open questions
/// assumption — "the width of the text" is ambiguous once the heading itself
/// wraps at narrow widths).
fn heading_underline_width(spans: &[Span], content_width: usize) -> usize {
    let text: String = spans.iter().map(|s| sanitize(&s.text)).collect();
    UnicodeWidthStr::width(text.as_str()).min(content_width)
}

/// Turns a `Document` into printable `Line`s at `terminal_width`.
/// `content_width = terminal_width.min(100)`, clamped to a minimum of 1 so
/// pathologically narrow terminals never cause a division/loop issue.
/// Inserts exactly one blank `Line` between every adjacent pair of top-level
/// blocks (decided, uniform — see plan Open questions); never a leading or
/// trailing blank line.
pub fn wrap(document: &Document, terminal_width: usize) -> LayoutResult {
    let content_width = terminal_width.clamp(1, 100);
    let mut lines: Vec<Line> = Vec::new();
    let mut block_start_line: Vec<usize> = Vec::with_capacity(document.blocks.len());

    for (i, block) in document.blocks.iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
        }
        block_start_line.push(lines.len());

        match block {
            Block::Heading { level, spans } => {
                let block_start = lines.len();
                let presented = heading_presentation_spans(*level, spans);
                lines.extend(wrap_spans(&presented, content_width));
                if *level == 1 || *level == 2 {
                    let underline_width = heading_underline_width(spans, content_width);
                    let ch = if *level == 1 { '═' } else { '─' };
                    lines.push(Line {
                        spans: vec![Span {
                            text: ch.to_string().repeat(underline_width),
                            style: Style {
                                bold: true,
                                fg: Some(style::HEADING),
                                ..Style::default()
                            },
                        }],
                    });
                }
                // A heading with no inline content (e.g. `###` alone) would
                // otherwise contribute zero lines, leaving this heading's
                // `block_start_line` entry pointing past its own block —
                // out of bounds if it's the last block, or at the next
                // block's line otherwise. Every heading must own at least
                // one (possibly blank) line.
                if lines.len() == block_start {
                    lines.push(Line::default());
                }
            }
            Block::Paragraph { spans } => {
                lines.extend(wrap_spans(spans, content_width));
            }
            Block::Rule => {
                lines.push(Line {
                    spans: vec![Span {
                        text: "─".repeat(content_width),
                        style: Style {
                            dim: true,
                            ..Style::default()
                        },
                    }],
                });
            }
            // Temporary: real rendering for these lands with the recursive
            // `wrap_block` refactor (Phase 3 plan task 8). Until then they
            // contribute no lines, same as being silently skipped upstream
            // in `render.rs` pre-Phase-3.
            Block::CodeBlock { .. }
            | Block::BlockQuote { .. }
            | Block::List { .. }
            | Block::Html { .. }
            | Block::FootnoteDef { .. } => {}
        }
    }

    let heading_lines = document
        .headings
        .iter()
        .map(|h| block_start_line[h.block_index])
        .collect();

    LayoutResult {
        lines,
        heading_lines,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::build_document;

    fn plain_spans(lines: &[Line]) -> Vec<String> {
        lines
            .iter()
            .map(|l| l.spans.iter().map(|s| s.text.as_str()).collect())
            .collect()
    }

    #[test]
    fn sanitize_strips_carriage_returns() {
        assert_eq!(sanitize("a\r\nb\rc"), "a\nbc");
    }

    #[test]
    fn sanitize_replaces_tabs_with_a_single_space() {
        assert_eq!(sanitize("a\tb"), "a b");
    }

    #[test]
    fn sanitize_replaces_controls_with_replacement_char() {
        assert_eq!(sanitize("a\u{0007}b\u{007F}c"), "a\u{FFFD}b\u{FFFD}c");
    }

    #[test]
    fn wraps_at_exact_width_boundary() {
        let doc = build_document("hello world");
        let result = wrap(&doc, 5);
        assert_eq!(plain_spans(&result.lines), vec!["hello", "world"]);
    }

    #[test]
    fn cjk_and_emoji_widths_are_respected() {
        // Each of 你/好 is display-width 2; at content_width 3 only one fits
        // per line (2 + a space would be 3, but the second char needs 2 more
        // which doesn't fit in the remaining 1 column).
        let doc = build_document("你好 x");
        let result = wrap(&doc, 3);
        let rendered: Vec<String> = plain_spans(&result.lines);
        // No char is ever split in half: every line's text is valid.
        for line in &rendered {
            assert!(line.chars().all(|c| "你好 x".contains(c)));
        }
        assert_eq!(
            rendered
                .concat()
                .chars()
                .filter(|c| !c.is_whitespace())
                .count(),
            3
        );
    }

    #[test]
    fn style_is_preserved_when_a_bold_word_is_split_across_a_wrap() {
        let doc = build_document("**abcdefghij**");
        let result = wrap(&doc, 5);
        assert_eq!(result.lines.len(), 2);
        for line in &result.lines {
            assert_eq!(line.spans.len(), 1);
            assert!(line.spans[0].style.bold);
        }
        assert_eq!(plain_spans(&result.lines), vec!["abcde", "fghij"]);
    }

    #[test]
    fn hard_break_flushes_the_line_without_visible_content() {
        let doc = build_document("line one  \nline two");
        let result = wrap(&doc, 80);
        assert_eq!(plain_spans(&result.lines), vec!["line one", "line two"]);
    }

    #[test]
    fn two_hundred_char_unbroken_word_hard_breaks_at_width() {
        let word = "a".repeat(200);
        let doc = build_document(&word);
        let result = wrap(&doc, 100);
        assert_eq!(result.lines.len(), 2);
        assert_eq!(
            plain_spans(&result.lines),
            vec!["a".repeat(100), "a".repeat(100)]
        );
    }

    #[test]
    fn rule_becomes_a_full_width_dim_line() {
        let doc = build_document("---");
        let result = wrap(&doc, 10);
        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0].spans.len(), 1);
        assert_eq!(result.lines[0].spans[0].text, "─".repeat(10));
        assert!(result.lines[0].spans[0].style.dim);
    }

    #[test]
    fn sanitization_neutralizes_esc_byte_and_osc52_sequence() {
        let malicious = "before\u{001b}]52;c;BASE64DATA==\u{0007}after";
        let doc = build_document(malicious);
        let result = wrap(&doc, 80);
        for line in &result.lines {
            for span in &line.spans {
                assert!(!span.text.as_bytes().contains(&0x1b));
            }
        }
    }

    /// Hand-rolled linear congruential generator producing a deterministic,
    /// valid-UTF-8 pseudo-random string. No `rand` dependency (Section 2
    /// forbids extra crates) — values are folded into the printable ASCII
    /// range so the output is always valid UTF-8.
    fn lcg_pseudo_random_string(len: usize, seed: u64) -> String {
        let mut state = seed;
        let mut out = String::with_capacity(len);
        for _ in 0..len {
            state = state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            let byte = ((state >> 33) % 95) as u8 + 0x20; // printable ASCII range
            out.push(byte as char);
        }
        out
    }

    #[test]
    fn no_panic_on_adversarial_input_at_narrow_widths() {
        // corpus.md-based coverage is added in task 8's snapshot test, once
        // the corpus file exists; this covers the seeded-LCG half of the
        // Section 12 robustness requirement now.
        let adversarial = lcg_pseudo_random_string(10_000, 0xDEAD_BEEF);
        for width in [1, 2, 40] {
            let doc = build_document(&adversarial);
            let _ = wrap(&doc, width);
        }
    }

    #[test]
    fn h1_is_bold_heading_colored_uppercase_with_double_underline() {
        let doc = build_document("# hi");
        let result = wrap(&doc, 80);
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[0].spans[0].text, "HI");
        assert_eq!(result.lines[0].spans[0].style.bold, true);
        assert_eq!(result.lines[0].spans[0].style.fg, Some(style::HEADING));
        assert_eq!(result.lines[1].spans[0].text, "══");
    }

    #[test]
    fn h2_gets_single_line_underline() {
        let doc = build_document("## hi");
        let result = wrap(&doc, 80);
        assert_eq!(result.lines.len(), 2);
        assert_eq!(result.lines[1].spans[0].text, "──");
    }

    #[test]
    fn h3_is_bold_heading_colored_with_no_underline() {
        let doc = build_document("### hi");
        let result = wrap(&doc, 80);
        assert_eq!(result.lines.len(), 1);
        assert_eq!(result.lines[0].spans[0].style.bold, true);
        assert_eq!(result.lines[0].spans[0].style.fg, Some(style::HEADING));
    }

    #[test]
    fn h4_to_h6_are_bold_with_section_prefix_and_no_color() {
        for (markdown, expected_text) in [
            ("#### hi", "§ hi"),
            ("##### hi", "§ hi"),
            ("###### hi", "§ hi"),
        ] {
            let doc = build_document(markdown);
            let result = wrap(&doc, 80);
            assert_eq!(result.lines.len(), 1);
            let rendered: String = result.lines[0]
                .spans
                .iter()
                .map(|s| s.text.as_str())
                .collect();
            assert_eq!(rendered, expected_text);
            // Inter-word space spans carry a neutral default style (task 5),
            // so only the non-space text spans are asserted bold/uncolored.
            for span in result.lines[0]
                .spans
                .iter()
                .filter(|s| !s.text.trim().is_empty())
            {
                assert!(span.style.bold, "expected bold span, got {span:?}");
                assert!(span.style.fg.is_none(), "expected no color, got {span:?}");
            }
        }
    }

    #[test]
    fn exactly_one_blank_line_between_every_pair_of_top_level_blocks() {
        let doc = build_document("# H1\n\npara\n\n---\n\n## H2\n\npara2");
        let result = wrap(&doc, 80);
        let rendered = plain_spans(&result.lines);
        // H1 line, H1 underline, blank, para, blank, rule, blank, H2, H2
        // underline, blank, para2 — never two consecutive blanks.
        for pair in rendered.windows(2) {
            assert!(
                !(pair[0].is_empty() && pair[1].is_empty()),
                "two consecutive blank lines in {rendered:?}"
            );
        }
        assert!(!rendered.first().unwrap().is_empty(), "leading blank line");
        assert!(!rendered.last().unwrap().is_empty(), "trailing blank line");
        assert!(rendered.contains(&"para".to_string()));
        assert!(rendered.contains(&"para2".to_string()));
    }

    #[test]
    fn heading_lines_maps_toc_entries_to_first_wrapped_line() {
        let doc = build_document("# H1\n\npara\n\n## H2");
        let result = wrap(&doc, 80);
        assert_eq!(result.heading_lines.len(), 2);
        assert_eq!(result.lines[result.heading_lines[0]].spans[0].text, "H1");
        assert_eq!(result.lines[result.heading_lines[1]].spans[0].text, "H2");
    }

    #[test]
    fn empty_trailing_heading_gets_a_valid_in_bounds_heading_line() {
        // A heading with no inline content (e.g. `###` alone) would
        // otherwise contribute zero lines; as the last block in the
        // document that left heading_lines pointing one past the end of
        // `lines`, an out-of-bounds index (adversarial-review finding).
        let doc = build_document("text\n\n###");
        let result = wrap(&doc, 80);
        assert_eq!(result.heading_lines.len(), 1);
        let index = result.heading_lines[0];
        assert!(
            index < result.lines.len(),
            "heading_lines[0] = {index} is out of bounds for {} lines",
            result.lines.len()
        );
        assert!(result.lines[index].spans.is_empty());
    }
}
