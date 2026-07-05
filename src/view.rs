use crate::layout::Line;

/// Viewport state: tracks which line is scrolled to the top and clamps all
/// movement to valid bounds. `set_layout` updates `lines`/`viewport_height`
/// together on a terminal resize, re-clamping `offset` to the new bounds.
pub struct ViewState {
    offset: usize,
    lines: Vec<Line>,
    viewport_height: usize,
}

impl ViewState {
    pub fn new(lines: Vec<Line>, viewport_height: usize) -> Self {
        ViewState {
            offset: 0,
            lines,
            viewport_height,
        }
    }

    /// The largest valid `offset` — the one that puts the last line at the
    /// bottom of the viewport.
    pub fn max_offset(&self) -> usize {
        self.lines.len().saturating_sub(self.viewport_height)
    }

    pub fn offset(&self) -> usize {
        self.offset
    }

    /// The lines currently visible in the viewport, in order.
    pub fn visible_lines(&self) -> &[Line] {
        let end = (self.offset + self.viewport_height).min(self.lines.len());
        &self.lines[self.offset..end]
    }

    /// Replaces the current layout (new wrapped lines + new viewport height)
    /// after a terminal resize, clamping `offset` so it never points past
    /// the new `max_offset`.
    pub fn set_layout(&mut self, lines: Vec<Line>, viewport_height: usize) {
        self.lines = lines;
        self.viewport_height = viewport_height;
        self.offset = self.offset.min(self.max_offset());
    }

    fn half_page(&self) -> usize {
        self.viewport_height / 2
    }

    pub fn scroll_down(&mut self, n: usize) {
        self.offset = self.offset.saturating_add(n).min(self.max_offset());
    }

    pub fn scroll_up(&mut self, n: usize) {
        self.offset = self.offset.saturating_sub(n);
    }

    pub fn half_page_down(&mut self) {
        let n = self.half_page();
        self.scroll_down(n);
    }

    pub fn half_page_up(&mut self) {
        let n = self.half_page();
        self.scroll_up(n);
    }

    pub fn jump_to_top(&mut self) {
        self.offset = 0;
    }

    pub fn jump_to_bottom(&mut self) {
        self.offset = self.max_offset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lines(n: usize) -> Vec<Line> {
        (0..n)
            .map(|i| Line {
                spans: vec![crate::style::Span {
                    text: format!("line {i}"),
                    style: crate::style::Style::default(),
                }],
            })
            .collect()
    }

    fn line_text(line: &Line) -> String {
        line.spans.iter().map(|s| s.text.as_str()).collect()
    }

    #[test]
    fn empty_file_has_zero_max_offset_and_no_visible_lines() {
        let view = ViewState::new(Vec::new(), 10);
        assert_eq!(view.max_offset(), 0);
        assert_eq!(view.offset(), 0);
        assert!(view.visible_lines().is_empty());
    }

    #[test]
    fn single_line_file() {
        let view = ViewState::new(lines(1), 10);
        assert_eq!(view.max_offset(), 0);
        assert_eq!(view.visible_lines().len(), 1);
        assert_eq!(line_text(&view.visible_lines()[0]), "line 0");
    }

    #[test]
    fn file_shorter_than_viewport_has_zero_max_offset() {
        let view = ViewState::new(lines(5), 10);
        assert_eq!(view.max_offset(), 0);
        assert_eq!(view.visible_lines().len(), 5);
    }

    #[test]
    fn scroll_down_is_clamped_at_max_offset() {
        let mut view = ViewState::new(lines(20), 10);
        assert_eq!(view.max_offset(), 10);
        view.scroll_down(100);
        assert_eq!(view.offset(), 10);
        assert_eq!(view.visible_lines().len(), 10);
    }

    #[test]
    fn scroll_up_is_clamped_at_zero() {
        let mut view = ViewState::new(lines(20), 10);
        view.scroll_up(100);
        assert_eq!(view.offset(), 0);
    }

    #[test]
    fn line_scrolling_moves_by_one() {
        let mut view = ViewState::new(lines(20), 10);
        view.scroll_down(1);
        assert_eq!(view.offset(), 1);
        view.scroll_up(1);
        assert_eq!(view.offset(), 0);
    }

    #[test]
    fn half_page_scrolling() {
        let mut view = ViewState::new(lines(100), 20);
        view.half_page_down();
        assert_eq!(view.offset(), 10);
        view.half_page_down();
        assert_eq!(view.offset(), 20);
        view.half_page_up();
        assert_eq!(view.offset(), 10);
    }

    #[test]
    fn jump_to_top_and_jump_to_bottom() {
        let mut view = ViewState::new(lines(50), 10);
        view.jump_to_bottom();
        assert_eq!(view.offset(), 40);
        view.jump_to_top();
        assert_eq!(view.offset(), 0);
    }

    #[test]
    fn set_layout_clamps_offset_to_the_new_max_offset() {
        let mut view = ViewState::new(lines(20), 10);
        view.jump_to_bottom();
        assert_eq!(view.offset(), 10);
        // Shrinking to 5 lines total drops max_offset to 0 (viewport height
        // unchanged); offset must be pulled back down to fit.
        view.set_layout(lines(5), 10);
        assert_eq!(view.offset(), 0);
        assert_eq!(view.visible_lines().len(), 5);
    }

    #[test]
    fn zero_viewport_height_never_panics() {
        let mut view = ViewState::new(lines(5), 0);
        assert_eq!(view.max_offset(), 5);
        view.half_page_down();
        assert_eq!(view.offset(), 0);
        view.jump_to_bottom();
        assert_eq!(view.offset(), 5);
        assert!(view.visible_lines().is_empty());
    }
}
