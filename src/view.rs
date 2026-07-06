use crate::layout::Line;

/// Section 6 interaction mode. `SearchInput` and `Toc` override Normal-mode
/// keybindings (see `input::map`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Mode {
    #[default]
    Normal,
    SearchInput,
    Toc,
}

/// The last *executed* search (Enter was pressed on a non-empty query with at
/// least one match). Distinct from the in-progress `ViewState::search_input`
/// buffer that accumulates while typing in `SearchInput` mode — an
/// unsubmitted or empty-submitted query never touches this.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct SearchState {
    pub query: String,
    /// Line indices into `ViewState::lines` (post-wrap) whose flattened plain
    /// text contains `query`, case-insensitively.
    pub matches: Vec<usize>,
    /// Index into `matches` of the currently-jumped-to match.
    pub current: usize,
}

/// Viewport state: tracks which line is scrolled to the top and clamps all
/// movement to valid bounds. `set_layout` updates `lines`/`viewport_height`
/// together on a terminal resize, re-clamping `offset` to the new bounds and
/// resetting all Phase 5 interaction state (mode, search, in-progress query)
/// back to a simple `Normal`/empty baseline — see the Phase 5 plan's Resolved
/// design decisions for why resize doesn't try to preserve them.
pub struct ViewState {
    offset: usize,
    lines: Vec<Line>,
    viewport_height: usize,
    heading_lines: Vec<usize>,
    mode: Mode,
    search: Option<SearchState>,
    search_input: String,
    toc_cursor: usize,
    toc_scroll: usize,
    status_message: Option<String>,
}

impl ViewState {
    pub fn new(lines: Vec<Line>, heading_lines: Vec<usize>, viewport_height: usize) -> Self {
        ViewState {
            offset: 0,
            lines,
            viewport_height,
            heading_lines,
            mode: Mode::Normal,
            search: None,
            search_input: String::new(),
            toc_cursor: 0,
            toc_scroll: 0,
            status_message: None,
        }
    }

    pub fn mode(&self) -> Mode {
        self.mode
    }

    pub fn search(&self) -> Option<&SearchState> {
        self.search.as_ref()
    }

    /// The in-progress query buffer while `mode() == Mode::SearchInput`.
    /// Meaningless in any other mode.
    pub fn search_input(&self) -> &str {
        &self.search_input
    }

    pub fn toc_cursor(&self) -> usize {
        self.toc_cursor
    }

    pub fn toc_scroll(&self) -> usize {
        self.toc_scroll
    }

    pub fn heading_lines(&self) -> &[usize] {
        &self.heading_lines
    }

    pub fn status_message(&self) -> Option<&str> {
        self.status_message.as_deref()
    }

    pub fn set_status_message(&mut self, message: String) {
        self.status_message = Some(message);
    }

    pub fn clear_status_message(&mut self) {
        self.status_message = None;
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
    /// the new `max_offset`. Also forces `mode` back to `Normal` and drops
    /// any active/in-progress search, regardless of what mode this was
    /// called from — re-running a search or TOC overlay against relaid-out
    /// text is out of scope (Phase 5 plan, Resolved design decisions).
    pub fn set_layout(&mut self, lines: Vec<Line>, heading_lines: Vec<usize>, viewport_height: usize) {
        self.lines = lines;
        self.viewport_height = viewport_height;
        self.heading_lines = heading_lines;
        self.offset = self.offset.min(self.max_offset());
        self.mode = Mode::Normal;
        self.search = None;
        self.search_input.clear();
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
        let view = ViewState::new(Vec::new(), Vec::new(), 10);
        assert_eq!(view.max_offset(), 0);
        assert_eq!(view.offset(), 0);
        assert!(view.visible_lines().is_empty());
    }

    #[test]
    fn single_line_file() {
        let view = ViewState::new(lines(1), Vec::new(), 10);
        assert_eq!(view.max_offset(), 0);
        assert_eq!(view.visible_lines().len(), 1);
        assert_eq!(line_text(&view.visible_lines()[0]), "line 0");
    }

    #[test]
    fn file_shorter_than_viewport_has_zero_max_offset() {
        let view = ViewState::new(lines(5), Vec::new(), 10);
        assert_eq!(view.max_offset(), 0);
        assert_eq!(view.visible_lines().len(), 5);
    }

    #[test]
    fn scroll_down_is_clamped_at_max_offset() {
        let mut view = ViewState::new(lines(20), Vec::new(), 10);
        assert_eq!(view.max_offset(), 10);
        view.scroll_down(100);
        assert_eq!(view.offset(), 10);
        assert_eq!(view.visible_lines().len(), 10);
    }

    #[test]
    fn scroll_up_is_clamped_at_zero() {
        let mut view = ViewState::new(lines(20), Vec::new(), 10);
        view.scroll_up(100);
        assert_eq!(view.offset(), 0);
    }

    #[test]
    fn line_scrolling_moves_by_one() {
        let mut view = ViewState::new(lines(20), Vec::new(), 10);
        view.scroll_down(1);
        assert_eq!(view.offset(), 1);
        view.scroll_up(1);
        assert_eq!(view.offset(), 0);
    }

    #[test]
    fn half_page_scrolling() {
        let mut view = ViewState::new(lines(100), Vec::new(), 20);
        view.half_page_down();
        assert_eq!(view.offset(), 10);
        view.half_page_down();
        assert_eq!(view.offset(), 20);
        view.half_page_up();
        assert_eq!(view.offset(), 10);
    }

    #[test]
    fn jump_to_top_and_jump_to_bottom() {
        let mut view = ViewState::new(lines(50), Vec::new(), 10);
        view.jump_to_bottom();
        assert_eq!(view.offset(), 40);
        view.jump_to_top();
        assert_eq!(view.offset(), 0);
    }

    #[test]
    fn set_layout_clamps_offset_to_the_new_max_offset() {
        let mut view = ViewState::new(lines(20), Vec::new(), 10);
        view.jump_to_bottom();
        assert_eq!(view.offset(), 10);
        // Shrinking to 5 lines total drops max_offset to 0 (viewport height
        // unchanged); offset must be pulled back down to fit.
        view.set_layout(lines(5), Vec::new(), 10);
        assert_eq!(view.offset(), 0);
        assert_eq!(view.visible_lines().len(), 5);
    }

    #[test]
    fn zero_viewport_height_never_panics() {
        let mut view = ViewState::new(lines(5), Vec::new(), 0);
        assert_eq!(view.max_offset(), 5);
        view.half_page_down();
        assert_eq!(view.offset(), 0);
        view.jump_to_bottom();
        assert_eq!(view.offset(), 5);
        assert!(view.visible_lines().is_empty());
    }

    #[test]
    fn new_view_starts_in_normal_mode_with_no_search_or_status_message() {
        let view = ViewState::new(lines(5), vec![0, 2], 10);
        assert_eq!(view.mode(), Mode::Normal);
        assert!(view.search().is_none());
        assert_eq!(view.search_input(), "");
        assert_eq!(view.toc_cursor(), 0);
        assert_eq!(view.toc_scroll(), 0);
        assert_eq!(view.heading_lines(), &[0, 2]);
        assert!(view.status_message().is_none());
    }

    #[test]
    fn status_message_can_be_set_and_cleared() {
        let mut view = ViewState::new(lines(5), Vec::new(), 10);
        view.set_status_message("Pattern not found: xyz".to_string());
        assert_eq!(view.status_message(), Some("Pattern not found: xyz"));
        view.clear_status_message();
        assert!(view.status_message().is_none());
    }

    #[test]
    fn set_layout_resets_mode_search_and_search_input_regardless_of_prior_mode() {
        let mut view = ViewState::new(lines(20), vec![0], 10);
        // Directly poke private fields (test submodule shares module
        // visibility) to simulate mid-search state without depending on
        // task 2/4 methods that don't exist yet at this point in the plan.
        view.mode = Mode::SearchInput;
        view.search_input.push('x');
        view.search = Some(SearchState {
            query: "old".to_string(),
            matches: vec![1, 2],
            current: 0,
        });
        assert_eq!(view.mode(), Mode::SearchInput);
        assert_eq!(view.search_input(), "x");

        view.set_layout(lines(20), vec![0, 5], 10);
        assert_eq!(view.mode(), Mode::Normal);
        assert!(view.search().is_none());
        assert_eq!(view.search_input(), "");
        assert_eq!(view.heading_lines(), &[0, 5]);
    }
}
