# PLAN: Phase 5 — Search and TOC

_Branch:_ `feature/issue-7-phase5-search-and-toc`
_Date:_ 2026-07-06
_Status:_ IN PROGRESS
_Source:_ #7
_PR:_ #<pr-number>
<!-- filled in after first push; omit until then -->

---

## Problem

Implement full Section 6 interaction behavior on top of the existing Phase 1-4
rendering pipeline: incremental substring search with match highlighting
(`/`, `n`/`N`), a centered TOC overlay (`t`), and the Section 8 status bar
(currently not implemented at all — deferred by both Phase 1 and Phase 2
plans specifically to land here). This is the last phase before v1
acceptance (Phase 6 syntax highlighting is optional/out of scope).

The data this phase consumes already exists: `Document.headings:
Vec<TocEntry>` (render.rs) and `LayoutResult.heading_lines: Vec<usize>`
(layout.rs) were built in earlier phases as byproducts of heading parsing,
specifically for this. `Style.reverse` is reserved and unused, specifically
for the search highlight. Nothing in `view.rs`, `input.rs`, or `main.rs`
currently references modes, search, or TOC.

## Out of scope

- Phase 6 (syntax highlighting, `syntect`) — not part of v1, not touched.
- Any dependency beyond the 3 named in the build plan (`pulldown-cmark`,
  `crossterm`, `unicode-width`).
- Regex or fuzzy search — Section 6 specifies plain case-insensitive
  substring match only.
- Search matches spanning a wrapped-line boundary. `SearchState.matches` is
  defined in Section 6 as line indices into the already-*wrapped*
  `ViewState.lines`, and matching is against "each Line's concatenated plain
  text" — so a query that only appears split across a wrap point (e.g.
  "hello" at the end of one wrapped line, "world" at the start of the next)
  will not match. This is inherent to the spec's data model, not a bug to
  work around.

## Impact assessment

- `src/view.rs` — the main change. Add `Mode`, `SearchState`, extend
  `ViewState` with `mode`, `search`, `toc_cursor`, `toc_scroll`,
  `heading_lines`, `status_message` fields and the methods to drive them
  (enter/exit SearchInput, execute search, cycle matches, open/navigate/
  close TOC, jump to a heading). `set_layout` (the resize path) must also
  reset non-Normal state — see Resolved design decisions below.
- `src/input.rs` — extend `Action` with the Phase 5 variants (`ToggleToc`,
  `StartSearch`, `NextMatch`, `PrevMatch`, `Escape`, `TocUp`, `TocDown`,
  `TocJump`, plus SearchInput's character-level actions: `SearchChar(char)`,
  `SearchBackspace`, `SearchExecute`). Mapping becomes mode-dependent, so
  `map` needs the current `Mode` as an input (Section 6: "SearchInput and
  Toc modes override keys").
- `src/main.rs` — new: reserve one row for the status bar in both
  `ViewState` construction sites; render the Section 8 status bar (or the
  transient `status_message`, or the `/query▌` input line in
  `Mode::SearchInput`) as that row every draw; render the TOC overlay box
  on top of content lines when `Mode::Toc`. Event loop dispatches through
  the mode-aware `input::map` and clears `status_message` on every
  processed keypress.
- `src/style.rs` — no struct changes needed (`reverse` already exists);
  only its doc comment ("nothing sets it yet") becomes stale once search
  highlighting lands.
- `src/render.rs`, `src/layout.rs` — no changes anticipated; both already
  expose what this phase needs (`TocEntry`, `heading_lines`).
- Risk: this is the first phase that has to compose multiple render layers
  (content + status bar + optional overlay) in one draw, and the first
  mode-dependent input dispatch — the highest-complexity phase so far
  relative to the others.

## Resolved design decisions (from Assess-phase clarification)

- **Resize during active search** → drop `SearchState` entirely, return to
  `Mode::Normal`. Simpler than re-running the scan against relaid-out text;
  matches disappearing on resize is an acceptable, simple contract.
- **Resize while TOC overlay is open** → close the overlay, return to
  `Mode::Normal`, rather than recomputing `heading_lines`/`toc_cursor`
  against the new layout.
- Net rule: `ViewState::set_layout` always forces `mode = Mode::Normal` and
  clears `search`, regardless of what mode it was called from. (This also
  covers resize while typing in `SearchInput` — the in-progress query is
  discarded, same simple contract.)
- **`q` while TOC overlay is open** → closes the overlay (treated as an
  unwritten but intentional override), rather than quitting the app per the
  literal Section 7 table.
- **Empty query submitted in `SearchInput`** (`/` then Enter) → no-op, back
  to `Mode::Normal`. Does not execute a search, does not touch any existing
  `SearchState` from a previous search.
- **Ctrl-C** — assumed to quit immediately regardless of mode (safety
  escape hatch); low-risk, reversible if wrong.
- **TOC cursor movement at the list boundary** (`j`/`k`/arrows past the
  first/last heading) → clamp, do not wrap. Consistent with every other
  navigation in the app (Normal-mode scrolling clamps at `max_offset`,
  never wraps). Only search `n`/`N` wraps — that's explicit in Section 6
  and the Phase 5 acceptance criteria; nothing says TOC cursor movement
  does.
- **TOC "scroll to keep selection visible"** is *stateful*, not
  recomputed from scratch each render: `ViewState` gets a `toc_scroll:
  usize` (top visible row index within the heading list) alongside
  `toc_cursor`. `toc_up`/`toc_down` adjust `toc_cursor` and then nudge
  `toc_scroll` only far enough to keep `toc_cursor` inside the visible
  window (classic clamp-scroll, not re-centering) — this belongs in
  `view.rs` (task 4), not computed ad hoc in `main.rs`'s render code.
- **TOC initial cursor position on open** → nearest current position:
  `toc_cursor` starts at the last heading whose `heading_lines[i] <=
  offset`, or `0` if the offset is before the first heading. `toc_scroll`
  is initialized so that heading is visible (e.g. equal to `toc_cursor`
  clamped so the window doesn't run off the end of the list).
- **Narrow-terminal / short-terminal rendering** (status bar row, TOC
  overlay box) → clip/truncate gracefully, never panic, never skip
  drawing. All width/height arithmetic in the new `main.rs` rendering
  code uses `saturating_sub`/`.min()`, matching the existing
  `layout.rs`/`render.rs` precedent (`no_panic_on_adversarial_input_at_narrow_widths`,
  `corpus_renders_without_panicking_at_narrow_widths`). Truncation
  priority when the status bar's left (filename) + right
  (percentage/counts/hint) text together exceed `terminal_width`: shrink/
  drop the right-hand hint (`t:toc /:search q:quit`) first, then the
  percentage/count block, before ever truncating the filename. The TOC
  box's heading-text truncation (`…`) already handles overflow within a
  fixed box width; if `terminal_width - 4` or `terminal_height - 4`
  computes to 0 or would underflow, clamp to a minimum of e.g. 1 rather
  than underflowing a `usize`.
- **Status row reservation**: the content viewport must reserve exactly 1
  row for the status bar (or `/query▌` input line) at all times,
  including while a TOC overlay is drawn on top of content. This means
  every call site that currently constructs/updates `ViewState` with the
  raw terminal height (`ViewState::new` in `run`, `set_layout` on
  resize) must pass `height.saturating_sub(1)` instead of `height`, not
  just the status-bar percentage formula in isolation — this is a real
  code change to two call sites in `main.rs`, not only a formula detail.
  `saturating_sub` avoids a panic at `height == 0`.
- **Current/total line display** (`120/284` in the Section 8 example) is
  1-indexed: `offset + 1` over `lines.len()`, not the raw 0-indexed
  `offset`.
- **`n`/`N` with no active search** (`view.search` is `None`) → silent
  no-op, consistent with "any key not listed is ignored silently."
- **Search highlight styling** does not visually distinguish the
  "current" match from other matches on screen — Section 6 defines a
  single `reverse` style for "all matches on visible lines"; do not
  invent a second highlight style for the current one.
- **"Until the next keypress"** (transient status messages: `Pattern not
  found: query`, `No headings`) is cleared by the next `Event::Key` with
  `KeyEventKind::Press`, whether or not it maps to a recognized `Action`
  — not only by the next *recognized* action. This needs a shared
  `status_message: Option<String>` (or similarly-named) field on
  `ViewState`, set by the search-no-match and no-headings paths and
  cleared unconditionally at the top of the main loop's key-handling
  branch before dispatching the new key.

## Open questions

None blocking — all identified ambiguities were resolved above before
writing this plan.

---

## Tasks

### Implementation

- [ ] 0. Create branch `feature/issue-7-phase5-search-and-toc` from develop
      following docs/git-workflow.md
- [x] 1. `view.rs`: add `Mode` enum (`Normal`, `SearchInput`, `Toc`) and
      `SearchState` struct (`query: String, matches: Vec<usize>, current:
      usize`) per Section 6. Extend `ViewState` with `mode: Mode`, `search:
      Option<SearchState>`, `toc_cursor: usize`, `toc_scroll: usize`,
      `heading_lines: Vec<usize>`, `status_message: Option<String>`. Thread
      `heading_lines` through `ViewState::new` and `set_layout`
      (layout::wrap already returns it). `set_layout` resets `mode` to
      `Normal` and clears `search` (resolved decision above); it does not
      need to touch `status_message`. Unit tests for construction/
      resize-reset behavior.
- [x] 2. `view.rs`: search execution — case-insensitive substring scan over
      each `Line`'s concatenated plain text (helper to flatten a `Line`'s
      spans into a `String`), building `SearchState.matches`. On execute:
      empty query → no-op, return to `Mode::Normal`, do not touch any
      existing `search`/`status_message`; non-empty with no matches → set
      `status_message = Some(format!("Pattern not found: {query}"))`,
      return to `Mode::Normal`, leave `search` as `None`; non-empty with
      matches → jump to first match at or after current offset (by line
      index), wrapping to top if none, enter `Mode::Normal` with `search`
      populated. `n`/`N` cycle `current` forward/backward with wraparound
      and re-scroll so the match's line is visible; both are a silent
      no-op when `search` is `None`. Unit tests: match-at-cursor,
      wraparound in both directions, case-insensitivity, no-match path
      (status_message set correctly), empty-query no-op, n/N with no
      active search.
- [x] 3. `view.rs`: search highlight mapping — given a visible `Line` and
      the active query, split matched byte ranges of its flattened plain
      text back into span boundaries and mark the matched runs with
      `style.reverse = true` (splitting a `Span` in two/three as needed,
      preserving the rest of its style). Applies identically to *all*
      matches on a visible line, including the current one — Section 6
      defines one highlight style for every match; do not add a second,
      distinct style for the "current" match. Unit tests: match inside a
      single span, match crossing a span boundary (e.g. spanning into a
      bold run), multiple matches on one line.
- [x] 4. `view.rs`: TOC navigation — `open_toc` sets `toc_cursor` to the
      last heading whose `heading_lines[i] <= offset` (or `0` if none
      qualify — i.e. the offset is before the first heading), initializes
      `toc_scroll` so that heading is within the visible window, and is a
      no-op when `heading_lines` is empty (caller sets `status_message =
      Some("No headings".into())`). `close_toc` returns to `Mode::Normal`.
      `toc_up`/`toc_down` move `toc_cursor` by one, **clamped** at
      `0`/`heading_lines.len().saturating_sub(1)` (no wraparound —
      consistent with Normal-mode scrolling elsewhere in the app; the
      `saturating_sub` is defensive — `toc_up`/`toc_down` should never
      actually be reachable with an empty `heading_lines` since `open_toc`
      guards that case, but the clamp must not panic if that invariant is
      ever violated), then adjust `toc_scroll`
      only enough to keep `toc_cursor` inside the caller-supplied visible
      row count (clamp-scroll, not re-centering). `toc_jump` sets `offset`
      so `heading_lines[toc_cursor]` is the top visible line, then returns
      to `Mode::Normal`. Unit tests: open lands on nearest-preceding
      heading (including the before-first-heading case), open on a
      headingless document is a no-op, up/down clamp at both ends,
      toc_scroll only moves when the cursor would leave the window, jump
      lands the heading at the top.
- [x] 5. `input.rs`: extend `Action` with `ToggleToc`, `StartSearch`,
      `NextMatch`, `PrevMatch`, `Escape`, `SearchChar(char)`,
      `SearchBackspace`, `SearchExecute`, `TocUp`, `TocDown`, `TocJump`
      (dedicated Toc actions, not reused `LineUp`/`LineDown` — those carry
      `ViewState` scrolling semantics, these carry `toc_cursor` semantics,
      and conflating them would require the caller to disambiguate by mode
      anyway). Change `map`'s signature to take the current `Mode` so it
      can dispatch correctly: in `Normal`, add `t`/`/`/`n`/`N`/`Esc`
      bindings alongside the existing Phase 1 set; in `SearchInput`,
      printable chars → `SearchChar`, Backspace → `SearchBackspace`, Enter
      → `SearchExecute`, Esc → `Escape`, Ctrl-C still → `Quit`; in `Toc`,
      `j`/`k`/arrows → `TocDown`/`TocUp`, Enter → `TocJump`, Esc/`t`/`q` →
      `Escape` (closes the overlay; per resolved decision `q` does not
      quit while Toc is open). Update existing tests for the new
      signature; add tests for each mode's overrides (e.g. `t` is literal
      in SearchInput, `q` closes rather than quits in Toc).
- [x] 6. `main.rs`: reserve the status row at both `ViewState` call sites —
      change `ViewState::new(layout_result.lines, height as usize)` in
      `run` and the `Event::Resize` handler's `view.set_layout(...)` call
      to pass `(height as usize).saturating_sub(1)` instead of the raw
      terminal height, so content and status bar never overlap and
      `height == 0` can't underflow.
- [x] 7. `main.rs`: status bar (Section 8) — render a reversed-video row as
      the last terminal row: file basename left-aligned; right-aligned
      `percentage% · current/total · t:toc /:search q:quit` where
      `current` is `offset + 1` (1-indexed) and `percentage` uses the
      exact Section 8 formula. If `status_message` is set, render it in
      place of the percentage/count/hint block instead (still right- or
      full-width per Section 6's "No headings"/"Pattern not found"
      wording — treat it the same slot as the normal right-hand text), and
      clear `status_message` at the top of the next processed
      `Event::Key(Press)` handling, regardless of whether that key maps to
      an `Action`. If filename + right-hand text together exceed
      `terminal_width`, drop the hint (`t:toc /:search q:quit`) first,
      then the percentage/count block, before ever truncating the
      filename — never panic on any width (use `saturating_sub`/`.min()`
      throughout, matching the `layout.rs` narrow-width precedent). Track/
      pass the basename from `RunConfig`/CLI args into `run`.
- [ ] 8. `main.rs`: TOC overlay rendering — centered box per Section 6's
      geometry (`width = min(60, terminal_width.saturating_sub(4)).max(1)`,
      `height = min(heading_count + 2, terminal_height.saturating_sub(4)).max(1)`
      — clamp both to a minimum of 1 rather than underflowing or drawing a
      zero-size box), single-line border characters, ` Contents ` title in
      the top border (truncated if the box is narrower than the title),
      2-space indent per level below H1, current selection in reverse,
      truncate overflowing heading text with a trailing `…`, and use
      `toc_scroll`/`toc_cursor` from task 4 to render the correct
      window of headings when there are more than fit.
- [ ] 9. `main.rs`: SearchInput rendering — replace the status bar row
      entirely with `/query▌`, truncating the query display (not the
      underlying `String`) from the left if it doesn't fit `terminal_width`
      so the cursor marker `▌` stays visible.
- [ ] 10. `main.rs`: wire the event loop through the mode-aware
      `input::map` and dispatch all new `Action` variants to the `view.rs`
      methods from tasks 2-4; redraw on every state-changing action (not
      just offset changes, since search/TOC/status_message state changes
      without moving `offset` too).
- [ ] 11. Extend `docs/mdv-build-plan.md`-referenced test corpus
      (`tests/corpus.md`) if needed so the corpus continues to exercise
      headings at multiple levels for TOC coverage (only if current corpus
      is insufficient — check before adding).

### Finish
- [ ] Write / update tests for all implementation tasks above
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected
- [ ] Convert draft PR to ready-for-review; add `Closes #7` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 7 --remove-label agent --remove-label "in progress" --add-label needs-review`

---
