use crate::layout::Line;
use crate::style::Span;

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
    pub fn set_layout(
        &mut self,
        lines: Vec<Line>,
        heading_lines: Vec<usize>,
        viewport_height: usize,
    ) {
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

    /// Scrolls the minimum amount needed to bring `line` into the viewport,
    /// without forcing it to any particular row — unlike `toc_jump`, which
    /// the Phase 5 plan pins to the top row explicitly, search's spec only
    /// says matches must be "visible" (initial jump) or "re-scrolled...
    /// visible" (`n`/`N`), so this preserves as much of the surrounding
    /// context as possible and keeps the `offset <= max_offset` invariant
    /// every other scroll method maintains.
    fn ensure_line_visible(&mut self, line: usize) {
        if line < self.offset {
            self.offset = line;
        } else if self.viewport_height > 0 && line >= self.offset + self.viewport_height {
            self.offset = (line + 1).saturating_sub(self.viewport_height);
        }
    }

    /// Enters `Mode::SearchInput` with an empty query buffer (`/` in Normal
    /// mode). Deliberately does not touch any existing `search` — Esc or an
    /// empty Enter must leave a previous search's highlights/matches intact.
    pub fn start_search(&mut self) {
        self.mode = Mode::SearchInput;
        self.search_input.clear();
    }

    pub fn search_push_char(&mut self, c: char) {
        self.search_input.push(c);
    }

    pub fn search_backspace(&mut self) {
        self.search_input.pop();
    }

    /// Cancels the in-progress query (Esc in `SearchInput`), discarding the
    /// buffer without touching any existing `search`.
    pub fn cancel_search_input(&mut self) {
        self.mode = Mode::Normal;
        self.search_input.clear();
    }

    /// Runs the in-progress query (Enter in `SearchInput`). Empty query is a
    /// no-op that leaves any existing `search`/`status_message` untouched; a
    /// non-empty query with no matches clears `search` and sets a transient
    /// `status_message`; a non-empty query with matches replaces `search`
    /// and scrolls to the first match at or after the current offset,
    /// wrapping to the top of the document if none qualify.
    pub fn execute_search(&mut self) {
        let query = std::mem::take(&mut self.search_input);
        self.mode = Mode::Normal;

        if query.is_empty() {
            return;
        }

        let needle = query.to_lowercase();
        let matches: Vec<usize> = self
            .lines
            .iter()
            .enumerate()
            .filter(|(_, line)| flatten_line(line).to_lowercase().contains(&needle))
            .map(|(i, _)| i)
            .collect();

        if matches.is_empty() {
            self.search = None;
            self.status_message = Some(format!("Pattern not found: {query}"));
            return;
        }

        let current = matches
            .iter()
            .position(|&line| line >= self.offset)
            .unwrap_or(0);
        let target_line = matches[current];
        self.ensure_line_visible(target_line);
        self.search = Some(SearchState {
            query,
            matches,
            current,
        });
    }

    /// Cycles to the next match, wrapping past the last back to the first.
    /// Silent no-op when there is no active search (Section 7: any key not
    /// listed is ignored silently — `n`/`N` with no search falls under it).
    pub fn next_match(&mut self) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        if search.matches.is_empty() {
            return;
        }
        search.current = (search.current + 1) % search.matches.len();
        let line = search.matches[search.current];
        self.ensure_line_visible(line);
    }

    /// Cycles to the previous match, wrapping past the first back to the
    /// last. Silent no-op when there is no active search.
    pub fn prev_match(&mut self) {
        let Some(search) = self.search.as_mut() else {
            return;
        };
        if search.matches.is_empty() {
            return;
        }
        search.current = if search.current == 0 {
            search.matches.len() - 1
        } else {
            search.current - 1
        };
        let line = search.matches[search.current];
        self.ensure_line_visible(line);
    }

    /// Esc's mode-dependent behavior (Section 7: "close overlay / cancel
    /// search input / clear search highlights"): closes the TOC overlay in
    /// `Toc` mode, discards the in-progress query in `SearchInput` mode
    /// (leaving any existing `search` untouched, same as an empty Enter), or
    /// clears active search highlights in `Normal` mode.
    pub fn escape(&mut self) {
        match self.mode {
            Mode::Toc => self.close_toc(),
            Mode::SearchInput => self.cancel_search_input(),
            Mode::Normal => self.search = None,
        }
    }

    /// Opens the TOC overlay (`t` in Normal mode). No-op when the document
    /// has no headings — the caller is responsible for surfacing a "No
    /// headings" status message in that case. Otherwise sets `toc_cursor` to
    /// the last heading at or before the current `offset` (`0` if the
    /// offset precedes every heading) and `toc_scroll` so that heading
    /// starts the visible window, unless it's close enough to the end of
    /// the list that doing so would leave blank rows below the last entry.
    /// `visible_rows` is the number of heading rows the TOC box can show at
    /// the terminal's current size (computed by the caller, not stored).
    pub fn open_toc(&mut self, visible_rows: usize) {
        if self.heading_lines.is_empty() {
            return;
        }
        self.toc_cursor = self
            .heading_lines
            .iter()
            .rposition(|&line| line <= self.offset)
            .unwrap_or(0);
        self.toc_scroll = self
            .toc_cursor
            .min(self.heading_lines.len().saturating_sub(visible_rows));
        self.mode = Mode::Toc;
    }

    pub fn close_toc(&mut self) {
        self.mode = Mode::Normal;
    }

    /// Nudges `toc_scroll` only far enough to keep `toc_cursor` inside a
    /// `visible_rows`-tall window (clamp-scroll, not re-centering).
    fn clamp_toc_scroll(&mut self, visible_rows: usize) {
        if visible_rows == 0 {
            return;
        }
        if self.toc_cursor < self.toc_scroll {
            self.toc_scroll = self.toc_cursor;
        } else if self.toc_cursor >= self.toc_scroll + visible_rows {
            self.toc_scroll = self.toc_cursor + 1 - visible_rows;
        }
    }

    /// Moves the TOC selection up one heading, clamped at the first entry
    /// (no wraparound, consistent with every other navigation in the app).
    pub fn toc_up(&mut self, visible_rows: usize) {
        self.toc_cursor = self.toc_cursor.saturating_sub(1);
        self.clamp_toc_scroll(visible_rows);
    }

    /// Moves the TOC selection down one heading, clamped at the last entry.
    /// The `saturating_sub(1)` is defensive: `open_toc` guards against an
    /// empty `heading_lines`, so this should never actually run against one,
    /// but must not panic if that invariant is ever violated.
    pub fn toc_down(&mut self, visible_rows: usize) {
        let last = self.heading_lines.len().saturating_sub(1);
        self.toc_cursor = (self.toc_cursor + 1).min(last);
        self.clamp_toc_scroll(visible_rows);
    }

    /// Jumps to the selected heading (Enter in `Toc` mode), making its line
    /// the top visible line, and returns to `Mode::Normal`. Uses `.get()`
    /// defensively; `toc_cursor` should always be in bounds by construction.
    pub fn toc_jump(&mut self) {
        if let Some(&line) = self.heading_lines.get(self.toc_cursor) {
            self.offset = line;
        }
        self.mode = Mode::Normal;
    }
}

/// Concatenates a `Line`'s spans into their plain text, for search matching.
fn flatten_line(line: &Line) -> String {
    line.spans.iter().map(|span| span.text.as_str()).collect()
}

/// Case-insensitive comparison key for one char. Takes only the first
/// `to_lowercase()` result rather than the full (possibly multi-char)
/// expansion — an approximation for the rare characters whose lowercase form
/// is more than one codepoint (e.g. Turkish İ), acceptable for a plain
/// substring search.
fn lower_char(c: char) -> char {
    c.to_lowercase().next().unwrap_or(c)
}

/// Finds every non-overlapping, case-insensitive occurrence of `query` in
/// `text`, returning byte ranges into `text` (not into any lowercased copy —
/// comparison happens char-by-char against the original `char_indices` so
/// the returned byte offsets always slice `text` cleanly, even though
/// lowercasing can change a character's UTF-8 byte length).
fn find_match_ranges(text: &str, query: &str) -> Vec<(usize, usize)> {
    let needle: Vec<char> = query.chars().map(lower_char).collect();
    if needle.is_empty() {
        return Vec::new();
    }

    let haystack: Vec<(usize, char)> = text
        .char_indices()
        .map(|(i, c)| (i, lower_char(c)))
        .collect();

    let mut ranges = Vec::new();
    let mut i = 0;
    while i + needle.len() <= haystack.len() {
        let is_match = (0..needle.len()).all(|k| haystack[i + k].1 == needle[k]);
        if is_match {
            let start = haystack[i].0;
            let end = haystack
                .get(i + needle.len())
                .map(|&(b, _)| b)
                .unwrap_or(text.len());
            ranges.push((start, end));
            i += needle.len();
        } else {
            i += 1;
        }
    }
    ranges
}

/// Maps every case-insensitive occurrence of `query` in `line`'s flattened
/// plain text back onto its spans, splitting a `Span` in two or three as
/// needed at match boundaries (including matches that cross a span
/// boundary) and setting `style.reverse = true` on the matched runs only.
/// Every match gets the identical highlight — Section 6 defines a single
/// style for all matches, current or not. Returns a clone of `line`
/// unchanged when `query` is empty or has no matches.
pub fn highlight_matches(line: &Line, query: &str) -> Line {
    if query.is_empty() {
        return line.clone();
    }

    let text = flatten_line(line);
    let ranges = find_match_ranges(&text, query);
    if ranges.is_empty() {
        return line.clone();
    }

    let mut new_spans = Vec::new();
    let mut span_start = 0usize;
    for span in &line.spans {
        let span_end = span_start + span.text.len();
        let mut cursor = span_start;

        for &(match_start, match_end) in &ranges {
            let overlap_start = match_start.max(span_start);
            let overlap_end = match_end.min(span_end);
            if overlap_start >= overlap_end {
                continue;
            }
            if cursor < overlap_start {
                new_spans.push(Span {
                    text: span.text[(cursor - span_start)..(overlap_start - span_start)]
                        .to_string(),
                    style: span.style,
                });
            }
            let mut style = span.style;
            style.reverse = true;
            new_spans.push(Span {
                text: span.text[(overlap_start - span_start)..(overlap_end - span_start)]
                    .to_string(),
                style,
            });
            cursor = overlap_end;
        }

        if cursor < span_end {
            new_spans.push(Span {
                text: span.text[(cursor - span_start)..].to_string(),
                style: span.style,
            });
        }

        span_start = span_end;
    }

    Line { spans: new_spans }
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

    fn text_lines(texts: &[&str]) -> Vec<Line> {
        texts
            .iter()
            .map(|t| Line {
                spans: vec![crate::style::Span {
                    text: t.to_string(),
                    style: crate::style::Style::default(),
                }],
            })
            .collect()
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

    fn typed(view: &mut ViewState, query: &str) {
        view.start_search();
        for c in query.chars() {
            view.search_push_char(c);
        }
    }

    #[test]
    fn start_search_enters_search_input_mode_without_touching_prior_search() {
        let mut view = ViewState::new(text_lines(&["alpha", "beta"]), Vec::new(), 10);
        typed(&mut view, "alpha");
        view.execute_search();
        assert!(view.search().is_some());

        view.start_search();
        assert_eq!(view.mode(), Mode::SearchInput);
        assert_eq!(view.search_input(), "");
        assert!(
            view.search().is_some(),
            "prior search must survive re-opening SearchInput"
        );
    }

    #[test]
    fn search_backspace_removes_last_char() {
        let mut view = ViewState::new(text_lines(&["alpha"]), Vec::new(), 10);
        typed(&mut view, "abc");
        view.search_backspace();
        assert_eq!(view.search_input(), "ab");
    }

    #[test]
    fn empty_query_execute_is_a_no_op() {
        let mut view = ViewState::new(text_lines(&["alpha", "beta"]), Vec::new(), 10);
        typed(&mut view, "beta");
        view.execute_search();
        let before = view.search().cloned();

        view.start_search();
        view.execute_search();
        assert_eq!(view.mode(), Mode::Normal);
        assert_eq!(
            view.search().cloned(),
            before,
            "empty submit must not touch existing search"
        );
        assert!(view.status_message().is_none());
    }

    #[test]
    fn cancel_search_input_discards_buffer_without_touching_existing_search() {
        let mut view = ViewState::new(text_lines(&["alpha", "beta"]), Vec::new(), 10);
        typed(&mut view, "alpha");
        view.execute_search();
        let before = view.search().cloned();

        view.start_search();
        view.search_push_char('z');
        view.cancel_search_input();
        assert_eq!(view.mode(), Mode::Normal);
        assert_eq!(view.search().cloned(), before);
    }

    #[test]
    fn non_empty_query_with_no_matches_sets_status_message_and_clears_search() {
        let mut view = ViewState::new(text_lines(&["alpha", "beta"]), Vec::new(), 10);
        typed(&mut view, "alpha");
        view.execute_search();
        assert!(view.search().is_some());

        typed(&mut view, "zzz");
        view.execute_search();
        assert_eq!(view.mode(), Mode::Normal);
        assert!(view.search().is_none());
        assert_eq!(
            view.status_message(),
            Some("Pattern not found: zzz"),
            "no-match search must clear any prior search highlights"
        );
    }

    #[test]
    fn search_is_case_insensitive() {
        let mut view = ViewState::new(text_lines(&["Alpha Beta"]), Vec::new(), 10);
        typed(&mut view, "ALPHA");
        view.execute_search();
        assert_eq!(view.search().unwrap().matches, vec![0]);
    }

    #[test]
    fn execute_search_jumps_to_first_match_at_or_after_offset() {
        let mut view = ViewState::new(
            text_lines(&["needle", "plain", "needle", "plain", "needle"]),
            Vec::new(),
            2,
        );
        view.scroll_down(3); // offset = 3
        typed(&mut view, "needle");
        view.execute_search();
        let search = view.search().unwrap();
        assert_eq!(search.matches, vec![0, 2, 4]);
        assert_eq!(search.current, 2, "line 4 is the first match >= offset 3");
        assert!(view.offset() <= 4 && view.offset() + 2 > 4);
    }

    #[test]
    fn execute_search_wraps_to_top_match_when_none_at_or_after_offset() {
        let mut view = ViewState::new(text_lines(&["needle", "plain", "plain"]), Vec::new(), 10);
        view.jump_to_bottom();
        typed(&mut view, "needle");
        view.execute_search();
        assert_eq!(view.search().unwrap().current, 0);
    }

    #[test]
    fn next_match_wraps_past_the_last_match() {
        let mut view = ViewState::new(text_lines(&["needle", "plain", "needle"]), Vec::new(), 10);
        typed(&mut view, "needle");
        view.execute_search();
        assert_eq!(view.search().unwrap().current, 0);
        view.next_match();
        assert_eq!(view.search().unwrap().current, 1);
        view.next_match();
        assert_eq!(
            view.search().unwrap().current,
            0,
            "wraps past the last match"
        );
    }

    #[test]
    fn prev_match_wraps_past_the_first_match() {
        let mut view = ViewState::new(text_lines(&["needle", "plain", "needle"]), Vec::new(), 10);
        typed(&mut view, "needle");
        view.execute_search();
        assert_eq!(view.search().unwrap().current, 0);
        view.prev_match();
        assert_eq!(
            view.search().unwrap().current,
            1,
            "wraps before the first match"
        );
    }

    #[test]
    fn next_and_prev_match_are_silent_no_ops_without_an_active_search() {
        let mut view = ViewState::new(text_lines(&["alpha"]), Vec::new(), 10);
        view.next_match();
        view.prev_match();
        assert!(view.search().is_none());
        assert_eq!(view.offset(), 0);
    }

    fn plain_span(text: &str) -> crate::style::Span {
        crate::style::Span {
            text: text.to_string(),
            style: crate::style::Style::default(),
        }
    }

    #[test]
    fn highlight_matches_marks_a_match_inside_a_single_span() {
        let line = Line {
            spans: vec![plain_span("hello world")],
        };
        let highlighted = highlight_matches(&line, "world");
        assert_eq!(
            highlighted.spans,
            vec![
                plain_span("hello "),
                crate::style::Span {
                    text: "world".to_string(),
                    style: crate::style::Style {
                        reverse: true,
                        ..Default::default()
                    },
                },
            ]
        );
    }

    #[test]
    fn highlight_matches_splits_a_match_crossing_a_span_boundary() {
        let bold = crate::style::Style {
            bold: true,
            ..Default::default()
        };
        let line = Line {
            spans: vec![
                plain_span("hello "),
                crate::style::Span {
                    text: "world".to_string(),
                    style: bold,
                },
            ],
        };
        // "lo wo" spans the boundary between "hello " (bytes 0..6) and
        // "world" (bytes 6..11).
        let highlighted = highlight_matches(&line, "lo wo");
        let expected_bold_reverse = crate::style::Style {
            bold: true,
            reverse: true,
            ..Default::default()
        };
        let plain_reverse = crate::style::Style {
            reverse: true,
            ..Default::default()
        };
        assert_eq!(
            highlighted.spans,
            vec![
                plain_span("hel"),
                crate::style::Span {
                    text: "lo ".to_string(),
                    style: plain_reverse,
                },
                crate::style::Span {
                    text: "wo".to_string(),
                    style: expected_bold_reverse,
                },
                crate::style::Span {
                    text: "rld".to_string(),
                    style: bold,
                },
            ]
        );
    }

    #[test]
    fn highlight_matches_marks_every_match_on_one_line_identically() {
        let line = Line {
            spans: vec![plain_span("needle stuff needle")],
        };
        let highlighted = highlight_matches(&line, "needle");
        let reverse = crate::style::Style {
            reverse: true,
            ..Default::default()
        };
        assert_eq!(
            highlighted.spans,
            vec![
                crate::style::Span {
                    text: "needle".to_string(),
                    style: reverse,
                },
                plain_span(" stuff "),
                crate::style::Span {
                    text: "needle".to_string(),
                    style: reverse,
                },
            ]
        );
    }

    #[test]
    fn highlight_matches_is_case_insensitive() {
        let line = Line {
            spans: vec![plain_span("Hello WORLD")],
        };
        let highlighted = highlight_matches(&line, "world");
        assert!(highlighted.spans[1].style.reverse);
        assert_eq!(line_text(&highlighted), "Hello WORLD");
    }

    #[test]
    fn highlight_matches_with_no_match_or_empty_query_returns_line_unchanged() {
        let line = Line {
            spans: vec![plain_span("hello world")],
        };
        assert_eq!(highlight_matches(&line, "").spans, line.spans);
        assert_eq!(highlight_matches(&line, "zzz").spans, line.spans);
    }

    #[test]
    fn open_toc_lands_on_the_nearest_preceding_heading() {
        let mut view = ViewState::new(lines(20), vec![0, 5, 10, 15], 10);
        view.scroll_down(7); // offset = 7, between headings at 5 and 10
        view.open_toc(10);
        assert_eq!(view.mode(), Mode::Toc);
        assert_eq!(view.toc_cursor(), 1); // heading_lines[1] == 5 <= 7
    }

    #[test]
    fn open_toc_before_the_first_heading_lands_on_index_zero() {
        let mut view = ViewState::new(lines(20), vec![5, 10], 10);
        assert_eq!(view.offset(), 0);
        view.open_toc(10);
        assert_eq!(view.toc_cursor(), 0);
    }

    #[test]
    fn open_toc_on_a_headingless_document_is_a_no_op() {
        let mut view = ViewState::new(lines(20), Vec::new(), 10);
        view.open_toc(10);
        assert_eq!(view.mode(), Mode::Normal);
    }

    #[test]
    fn toc_up_and_down_clamp_at_both_ends_without_wrapping() {
        let mut view = ViewState::new(lines(20), vec![0, 5, 10], 10);
        view.open_toc(10);
        assert_eq!(view.toc_cursor(), 0);
        view.toc_up(10);
        assert_eq!(
            view.toc_cursor(),
            0,
            "must clamp at the first entry, not wrap"
        );

        view.toc_down(10);
        view.toc_down(10);
        assert_eq!(view.toc_cursor(), 2);
        view.toc_down(10);
        assert_eq!(
            view.toc_cursor(),
            2,
            "must clamp at the last entry, not wrap"
        );
    }

    #[test]
    fn toc_scroll_only_moves_when_the_cursor_would_leave_the_window() {
        let heading_lines: Vec<usize> = (0..20).collect();
        let mut view = ViewState::new(lines(20), heading_lines, 10);
        view.open_toc(5); // 5 visible rows
        assert_eq!(view.toc_cursor(), 0);
        assert_eq!(view.toc_scroll(), 0);

        for _ in 0..4 {
            view.toc_down(5);
        }
        assert_eq!(view.toc_cursor(), 4);
        assert_eq!(view.toc_scroll(), 0, "cursor still inside the first window");

        view.toc_down(5);
        assert_eq!(view.toc_cursor(), 5);
        assert_eq!(
            view.toc_scroll(),
            1,
            "scroll nudges by exactly one, not re-centered"
        );

        for _ in 0..5 {
            view.toc_up(5);
        }
        assert_eq!(view.toc_cursor(), 0);
        assert_eq!(
            view.toc_scroll(),
            0,
            "scroll nudges back down once cursor leaves the top"
        );
    }

    #[test]
    fn toc_jump_makes_the_selected_heading_the_top_line_and_returns_to_normal() {
        let mut view = ViewState::new(lines(20), vec![0, 5, 10], 10);
        view.open_toc(10);
        view.toc_down(10);
        view.toc_jump();
        assert_eq!(view.mode(), Mode::Normal);
        assert_eq!(view.offset(), 5);
    }

    #[test]
    fn escape_closes_toc_cancels_search_input_or_clears_highlights_by_mode() {
        let mut view = ViewState::new(text_lines(&["needle", "plain"]), vec![0], 10);

        view.open_toc(10);
        view.escape();
        assert_eq!(view.mode(), Mode::Normal);

        view.start_search();
        view.search_push_char('x');
        view.escape();
        assert_eq!(view.mode(), Mode::Normal);
        assert_eq!(view.search_input(), "");

        typed(&mut view, "needle");
        view.execute_search();
        assert!(view.search().is_some());
        view.escape();
        assert!(
            view.search().is_none(),
            "Esc in Normal mode clears highlights"
        );
    }
}
