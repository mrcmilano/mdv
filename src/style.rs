use crossterm::style::Color;

/// H1–H3 heading color (build plan Section 5).
pub const HEADING: Color = Color::Cyan;
/// Inline code foreground (build plan Section 5).
pub const CODE: Color = Color::Yellow;
/// Link text foreground (build plan Section 5).
pub const LINK: Color = Color::Blue;
/// Blockquote gutter foreground (build plan Section 5: "Dim Green `┃ `").
pub const QUOTE: Color = Color::Green;
/// Checked task-list checkbox foreground (build plan Section 5).
pub const TASK_CHECKED: Color = Color::Green;

/// Terminal text style, restricted to crossterm's 16 ANSI colors (no RGB, per
/// Section 5) so the user's terminal theme applies. `reverse` is reserved for
/// the Phase 5 search-match highlight; nothing sets it yet.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Style {
    pub fg: Option<Color>,
    pub bold: bool,
    pub italic: bool,
    pub dim: bool,
    pub strikethrough: bool,
    pub reverse: bool,
    pub underline: bool,
}

/// A run of text with one style. `layout.rs` may split a `Span` across
/// multiple `Line`s when wrapping, but each resulting piece keeps this style
/// unchanged.
#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_style_is_all_off_and_no_color() {
        let style = Style::default();
        assert_eq!(style.fg, None);
        assert!(!style.bold);
        assert!(!style.italic);
        assert!(!style.dim);
        assert!(!style.strikethrough);
        assert!(!style.reverse);
        assert!(!style.underline);
    }
}
