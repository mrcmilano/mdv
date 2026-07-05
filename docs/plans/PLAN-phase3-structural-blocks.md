# PLAN: phase3-structural-blocks

_Branch:_ `feature/issue-5-phase3-structural-blocks`
_Date:_ 2026-07-05
_Status:_ IN PROGRESS
_Source:_ #5
_PR:_ #<pr-number>
<!-- filled in after first push; omit until then -->

---

## Problem

Implement build plan Section 9 Phase 3: code blocks, blockquotes (nested),
ordered/unordered/task lists (nested), raw HTML passthrough, and footnotes.
Phase 2 (`render.rs`/`layout.rs`) only handles inline content and three block
types (`Heading`, `Paragraph`, `Rule`); every other block-level pulldown-cmark
event is currently silently skipped via `skip_depth`. This phase adds the
remaining block types from Section 4's data model and their Section 5
rendering rules, including the recursive prefix-composition behavior nested
blocks require (e.g. a code block inside a blockquote).

## Out of scope

- Tables (`Block::Table`, `Alignment`) — Phase 4, issue #6.
- Search and TOC overlay — Phase 5.
- Syntax highlighting inside code blocks — Phase 6 (optional, feature-gated).
- Any change to the module layout (`docs/architecture.md`) — everything lands
  inside the existing `render.rs`/`layout.rs`.

## Impact assessment

- `src/render.rs`: add `Block::CodeBlock`, `Block::BlockQuote`, `Block::List`
  (+ a new `ListItem` struct), `Block::Html`, `Block::FootnoteDef`. The
  current flat single-pass event loop cannot build nested block trees
  (`BlockQuote`/`List` items containing arbitrary child blocks), so the
  block-level part of `build_document` becomes recursive-descent over the
  event stream; inline-span accumulation (styles, links, images) is reused
  unchanged inside that recursion.
- `src/layout.rs`: `wrap()`'s flat `match` over `Block` becomes a recursive
  `wrap_block` so nested blocks can recurse with a shrunk `content_width` and
  a composed line-prefix, per Section 5's "Prefix composition" rule. New
  per-line prefixing logic for code-block gutters, blockquote gutters
  (stacking on nesting), and list markers/checkboxes/indentation.
- `tests/corpus.md` / `tests/snapshots/corpus.txt`: extended with one
  instance of every new element (Section 10 requirement); snapshot
  regenerated via `UPDATE_SNAPSHOTS=1`.
- No change to `main.rs`, `input.rs`, `view.rs`, `style.rs`, or the module
  layout in `docs/architecture.md`.

## Open questions

None blocking. The build plan's Section 4/5 fully specify behavior; a few
implementation-level gaps are filled with the assumptions below (each is a
narrow reading of unambiguous adjacent spec text, not a product decision):

- Section 4's `List { items: Vec<Vec<Block>> }` has no field for a task
  item's checked state → ASSUMPTION: introduce
  `ListItem { checked: Option<bool>, blocks: Vec<Block> }` (`None` = not a
  task item) and use `items: Vec<ListItem>` instead of the literal
  `Vec<Vec<Block>>` shown in Section 4 (that section is illustrative, not a
  literal type to copy — `Table`'s own `Alignment` type isn't defined there
  either).
- Section 5 doesn't state Raw HTML's wrap behavior explicitly (only
  "Verbatim, Dim") → ASSUMPTION: treat it like a code block minus the
  gutter/language line — each source line printed dim, truncated (not
  wrapped/reflowed) with a trailing dim `…` if it overflows
  `content_width`, since "verbatim" implies no reflow and Code block is the
  nearest specified analogue.
- Nesting depth for list bullets/indent isn't carried in the `List` data
  model → ASSUMPTION: recursion depth in `layout::wrap_block` *is* the
  nesting level (a nested `Block::List` only ever appears as a child block
  inside a `ListItem`), so no extra field is needed on `Block::List` itself.
- "2-space indent per level" (inter-level nesting) vs. "continuation lines
  align under the text, not the bullet" (intra-item wrap) read as two
  separate rules, not one → ASSUMPTION: first line of an item gets prefix
  `(2 × level spaces) + marker + " "`; continuation lines (further wraps of
  that item's own content, and any subsequent blocks in a multi-block item)
  get prefix `(2 × level spaces) + " " × width(marker + " ")` so they align
  under the text. This resolves the apparent tension between a fixed
  2-space/level indent and variable-width ordinal markers (`1.` vs `10.`).
- Blockquote's "quoted text Dim" → ASSUMPTION: applied recursively to every
  span produced by a blockquote's contents (including nested blocks) by
  OR-ing `dim = true` onto each span's existing style when the `BlockQuote`
  closes in `render.rs`, rather than as a separate flag threaded through
  `layout.rs`. Keeps `layout.rs`'s prefixing logic style-agnostic.
- Footnote definitions: Section 5 explicitly states final placement
  ("rendered at the very end under a rule") regardless of source position —
  not an assumption, directly specified. `render.rs` collects
  `FootnoteDefinition` blocks in encounter order during the parse and
  appends `Rule` + one `FootnoteDef` per definition to `Document.blocks`
  after the main parse loop.

---

## Tasks

### Implementation

- [x] 0. Create branch `feature/issue-5-phase3-structural-blocks` from develop following docs/git-workflow.md
- [x] 1. `render.rs`: add the new `Block` variants and `ListItem` struct to the data model (no parsing changes yet). Existing Phase 1/2 tests must still pass unchanged.
- [x] 2. `render.rs`: refactor `build_document`'s block-level loop into recursive-descent (extract the existing inline-span accumulation — styles/links/images/text/breaks — into a reusable function called both at the top level and inside recursive block parsing). No behavior change for `Heading`/`Paragraph`/`Rule`/inline content; all existing tests pass unchanged.
- [x] 3. `render.rs`: implement `Tag::CodeBlock` → `Block::CodeBlock { language, lines }` (fenced + indented; concatenate `Text` events, split on `\n`, drop the single trailing empty line pulldown-cmark leaves). Unit tests: fenced with language, fenced without, indented.
- [x] 4. `render.rs`: implement `Tag::BlockQuote` → `Block::BlockQuote { blocks }`, recursive, forcing `dim = true` on every span within (including nested blocks) when it closes. Unit tests: single-level quote, 2-level nested quote, quote containing a code block.
- [x] 5. `render.rs`: implement `Tag::List`/`Tag::Item`/`Event::TaskListMarker` → `Block::List { ordered, items: Vec<ListItem> }`, recursive for nested lists. Unit tests: unordered, ordered with custom start number, task list (checked + unchecked), 3-level nested mixed list.
- [x] 6. `render.rs`: implement `Event::Html` (block) → `Block::Html { lines }` and `Event::InlineHtml` → dim `Span` appended into the current inline-accumulation buffer (same code path as inline `Code`). Unit tests: HTML block, inline HTML inside a paragraph.
- [x] 7. `render.rs`: implement `Tag::FootnoteDefinition` (collected out-of-band, recursive block parsing for its content) and `Event::FootnoteReference` → dim `[^label]` inline span; append `Rule` + `FootnoteDef` blocks to the end of `Document.blocks` after the main loop, in encounter order. Unit tests: reference + definition round trip, definition physically before its reference in source still renders at the end.
- [x] 8. `layout.rs`: introduce recursive `wrap_block(block, content_width) -> Vec<Line>` plus a prefix-composition helper (prepend a per-line prefix string, shrinking child `content_width` by the prefix's display width, clamped to a minimum of 1). Migrate `Heading`/`Paragraph`/`Rule` onto it with no output change — all existing snapshot/unit tests pass unchanged.
- [x] 9. `layout.rs`: `CodeBlock` rendering — `  │ ` dim gutter every content line, dim `(lang)` line right after the opening when present, sanitize() each source line same as inline text, truncate (never wrap) with a trailing dim `…` on overflow. Unit tests: no-wrap-at-narrow-width, language line present/absent, ESC-byte sanitization inside a code block.
- [x] 10. `layout.rs`: `BlockQuote` rendering — dim-green `┃ ` prefix per line, stacking outermost-first on nesting (`┃ ┃ `), recursing into child blocks at the shrunk width. Unit test: nested-quote-containing-code-block matches the spec's literal example `┃   │ code`.
- [x] 11. `layout.rs`: `List` rendering — bullet by level (`•`/`◦`/`▪`) or ordinal (`N.` from the item's own start number) or checkbox (`[✓]`/`[ ]`, green when checked) as the first-line marker; continuation-line padding of equal width; 2-space additional indent per nesting level. Unit tests: 3-level nested list continuation alignment (Section 9's Phase 3 acceptance criterion), ordered list with a 2-digit item number, task list checkbox colors.
- [x] 12. `layout.rs`: `Html` block rendering — dim verbatim lines, truncated like `CodeBlock` but without a gutter/language line. Unit test: overflow truncation + dim styling.
- [x] 13. `layout.rs`: `FootnoteDef` rendering — `[^label]: ` prefix on the first line, continuation lines padded to align under the content. Unit test: definition appears after a rule at document end, multi-line definition content aligns.
- [ ] 14. Extend `tests/corpus.md` with one instance of every new element (3-level nested list, nested blockquote, a code block with and without a language, raw HTML, task list, footnotes — per Section 10) and regenerate `tests/snapshots/corpus.txt` via `UPDATE_SNAPSHOTS=1`. Extend `corpus_renders_without_panicking_at_narrow_widths` coverage naturally (same corpus file, no code change needed there).

### Finish

- [ ] Write / update tests for all implementation tasks above
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected
- [ ] Convert draft PR to ready-for-review; add `Closes #5` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 5 --remove-label agent --remove-label "in progress" --add-label needs-review`

---

<!-- Add this section only if adversarial-review produced deferred LOW findings -->
## Deferred findings
<!-- Format: [LOW] <finding> — <rationale for deferral> — <follow-up issue if any> -->
