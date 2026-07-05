# PLAN: phase4-tables

_Branch:_ `feature/issue-6-phase4-tables`
_Date:_ 2026-07-05
_Status:_ IN PROGRESS
_Source:_ #6
_PR:_ #<pr-number>
<!-- filled in after first push; omit until then -->

---

## Problem

Implement build plan Section 9 Phase 4: box-drawing tables with alignment
and in-cell wrapping when the table is wider than the terminal. This is the
last structural element before Phase 5 (search/TOC) and completes
`Block::Table`, the one variant Section 4's data model lists that no phase
has implemented yet.

## Out of scope

- Search and TOC overlay — Phase 5.
- Syntax highlighting inside code blocks — Phase 6 (optional).
- Anything under build plan Section 1 "Non-goals".

## Impact assessment

- `src/render.rs`: add `Block::Table { header, rows, alignments }`. Parses
  `Tag::Table`/`TableHead`/`TableRow`/`TableCell` — per CommonMark's table
  grammar, cell content is inline-only (confirmed empirically: pulldown-cmark
  emits bare `Text`/`Strong`/etc. events directly inside `TableCell`, never a
  nested block), so this reuses the existing inline-accumulation logic
  (`ctx.current_spans`/`style_stack`/etc.) with no new recursion needed —
  simpler than `BlockQuote`/`List`.
- `src/layout.rs`: new column-width computation (natural width, then a
  shrink-widest-columns pass when the total overflows `content_width`),
  box-drawing border/junction rendering, alignment-aware cell padding, and
  in-cell wrapping (reusing `wrap_spans` at each column's final width).
- `tests/corpus.md` / `tests/snapshots/corpus.txt`: add a table wide enough
  to require shrinking, with all three alignment markers, per Section 10.
- No change to `main.rs`, `input.rs`, `view.rs`, `style.rs`, or the module
  layout in `docs/architecture.md`.

## Open questions

None blocking — Section 4/5 fully specify the data model and rendering
rules. Implementation-level gaps filled with these assumptions:

- Section 4 doesn't say where `Alignment` comes from → ASSUMPTION: reuse
  `pulldown_cmark::Alignment` (`None`/`Left`/`Center`/`Right`) directly as
  `Block::Table`'s alignment type rather than defining a redundant local
  enum — it's already exactly this shape and is what `Tag::Table` itself
  carries.
- Section 5 doesn't specify separator placement beyond listing the
  box-drawing character set → ASSUMPTION: top border (`┌─┬─┐`), header row,
  one separator row (`├─┼─┤`) directly below the header, all body rows
  un-separated, bottom border (`└─┴─┘`) — the standard convention (matches
  `glow`/GitHub's own table rendering), and CommonMark tables have exactly
  one header row so there's only one natural place for a separator.
- Section 5's "shrink the widest columns" isn't a precise algorithm →
  ASSUMPTION: iteratively decrement whichever column is currently widest by
  1 (ties broken by leftmost column) until the total fits `content_width`,
  floor each column at 1, then word-wrap each cell's text to its column's
  final width (reusing `wrap_spans`), growing the row's rendered height to
  the tallest cell and padding shorter cells with blank (space-filled)
  lines.
- Alignment padding: `None`/`Left` pad on the right, `Right` pads on the
  left, `Center` splits remaining space with any odd remainder going to the
  right — standard terminal-table convention, not spec-mandated but
  unambiguous once "respect column alignments" is read literally.

---

## Tasks

### Implementation

- [x] 0. Create branch `feature/issue-6-phase4-tables` from develop following docs/git-workflow.md
- [x] 1. `render.rs`: add `Block::Table { header: Vec<Vec<Span>>, rows: Vec<Vec<Vec<Span>>>, alignments: Vec<pulldown_cmark::Alignment> }` to the data model. Existing tests pass unchanged.
- [ ] 2. `render.rs`: implement `Tag::Table`/`TableHead`/`TableRow`/`TableCell` parsing — each cell's inline content accumulates via the existing `ctx.current_spans` machinery (same as a tight list item), flushed into that cell's `Vec<Span>` at `End(TableCell)`. Unit tests: header + one body row, alignment vector round-trip (`None`/`Left`/`Center`/`Right`), a cell containing styled inline content (bold/link/code).
- [ ] 3. `layout.rs`: natural column-width computation (max display-width per column across header + all rows, via `unicode_width`). Unit test: column widths match the widest cell in each column.
- [ ] 4. `layout.rs`: shrink-widest-columns pass when the natural total (plus border/padding overhead) exceeds `content_width`, floored at 1 per column, never panicking (test at width 40 per Section 9's acceptance criterion, and at width 1).
- [ ] 5. `layout.rs`: box-drawing rendering — top border, bold header row, one separator row, un-separated body rows, bottom border; alignment-aware padding; in-cell wrapping via `wrap_spans` at each column's final width, with row height growing to the tallest wrapped cell and shorter cells space-padded. Unit tests: alignment markers respected (`:---:` etc.), a table wider than the terminal shrinks/wraps without panicking at width 40, header row is bold, a cell's wrapped content stays inside its column's box.
- [ ] 6. Extend `tests/corpus.md` with a table wide enough to require shrinking and all three alignment markers (Section 10); regenerate `tests/snapshots/corpus.txt` via `UPDATE_SNAPSHOTS=1`.

### Finish

- [ ] Write / update tests for all implementation tasks above
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected
- [ ] Convert draft PR to ready-for-review; add `Closes #6` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 6 --remove-label agent --remove-label "in progress" --add-label needs-review`
      (superseded for this autonomous run — see AGENTS.md override in effect)

---

<!-- Add this section only if adversarial-review produced deferred LOW findings -->
## Deferred findings
<!-- Format: [LOW] <finding> — <rationale for deferral> — <follow-up issue if any> -->
