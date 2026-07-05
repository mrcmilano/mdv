# PLAN: Phase 2 — Rendering pipeline for inline + basic blocks

_Branch:_ `feature/issue-4-phase2-rendering`
_Date:_ 2026-07-05
_Status:_ READY
_Source:_ #4
_PR:_ #<pr-number>
<!-- filled in after first push -->

---

## Problem

Implement `docs/mdv-build-plan.md` **Section 9, Phase 2**: `render.rs` +
`layout.rs` turning parsed Markdown into printable, styled, wrapped lines for
headings, paragraphs, emphasis/strong/strikethrough, inline code, links,
images, horizontal rules, and hard/soft breaks, plus resize-triggered
re-layout. This replaces Phase 1's raw-line display — `main.rs` currently
sanitizes the file text and splits it into `Vec<String>` verbatim — with the
real Section 4 pipeline: `pulldown-cmark::Parser` → `render::build_document()`
→ `Document` → `layout::wrap(&document, width)` → `Vec<Line>` → `ViewState`.
Depends on Phase 1 (#1), merged to `develop` via PR #8.

## Out of scope

- `CodeBlock`, `BlockQuote`, `List` (ordered/unordered/task), `Table`, raw
  HTML passthrough, `FootnoteDef` — Phase 3 (structural blocks) and Phase 4
  (tables). The `Block` enum built this phase has only `Heading`,
  `Paragraph`, and `Rule` variants; the rest are added when their phases
  land. This mirrors Phase 1's precedent of building `ViewState` with only
  the fields that phase populated rather than the full Section 6 shape.
- Search, TOC, status bar, and the `mode`/`search`/`toc_cursor`/
  `heading_lines`-on-`ViewState` machinery — Phase 5. `Document.headings:
  Vec<TocEntry>` IS populated now (a free byproduct of visiting `Heading`
  blocks) but nothing consumes it until Phase 5.
- Syntax highlighting — Phase 6, optional, not built unless asked.
- Any change to `input.rs` keybindings.

## Impact assessment

New files: `src/style.rs` (`Style` + `Span` + fixed 16-color palette),
`src/render.rs` (`Block`/`Document`/`TocEntry` + `build_document()`),
`src/layout.rs` (`Line` + `wrap()` + sanitize, moved/extended from
`main.rs`'s Phase 1 `sanitize()`).

Modified: `src/main.rs` (add `mod style; mod render; mod layout;`; replace the
Phase 1 sanitize-then-split-lines codepath in `run()`; handle
`Event::Resize` by re-wrapping and clamping; `draw()` must print styled
`Span`s, not plain `String`s — currently the queried terminal width is
discarded as `_width`, this phase is where it starts being used).
`src/view.rs` (`ViewState.lines` retypes from `Vec<String>` to `Vec<Line>`).

**Spec correction (found in review):** `docs/mdv-build-plan.md` Section 4's
`Style` struct has no `underline` field, but Section 5 requires links render
"Blue underlined." Decided (user, 2026-07-05): add `underline: bool` to
`Style`. Task 1 below updates both `mdv-build-plan.md` Section 4 and
`docs/architecture.md` alongside the code change, on the feature branch (not
directly on `develop`, per the git-workflow hard rule) — this is why it's a
plan task rather than a pre-emptive doc fix.

New test assets: `tests/corpus.md` (Phase-2 elements only — later phases add
to it), `tests/snapshots/corpus.txt`.

Risk: per-span styling in `draw()` needs a color/attribute reset between
spans and after each line, or style bleeds onto unrelated content printed
afterward.

## Open questions

Two real spec gaps found in review were resolved directly with the user
(2026-07-05) and are folded into the tasks below rather than left open:
the `Style.underline` field (see Impact assessment), and block-to-block
blank-line spacing (see task 6 — uniform: exactly one blank `Line` between
every adjacent pair of top-level blocks).

Remaining assumptions (none blocking — see rationale, low-risk, revisit only
if implementation reveals a problem):

- Should the full `Block` enum (all 8 Section 4 variants, including
  `CodeBlock`/`List`/`Table`/etc.) be defined now, or only the 3 variants
  Phase 2 actually constructs? → ASSUMPTION: only `Heading`/`Paragraph`/
  `Rule` now — same incremental-build precedent Phase 1 set for
  `ViewState` (avoids dead code and speculative API surface; Section 4's
  enum is the eventual full shape, not a mandate to build it in one phase).
- The parser `Options` (Section 2: `ENABLE_TABLES`/`STRIKETHROUGH`/
  `TASKLISTS`/`FOOTNOTES`) are enabled from the start per spec, but Phase 2
  doesn't handle table/list/task/footnote/HTML events. What happens if
  `build_document()` receives one? → ASSUMPTION: any unhandled event —
  block-level tags (`Table`, `List`, `BlockQuote`, `CodeBlock`,
  `FootnoteDefinition`, `HtmlBlock`) as well as standalone leaf events
  (`Html`, `InlineHtml`, `TaskListMarker`, `FootnoteReference`) — is
  silently skipped: consumed so the flat iteration stays balanced, but
  contributes no `Block`/`Span`, never panics. Known, temporary limitation
  until Phase 3/4 add real handling; documented inline in `render.rs`; not
  exercised by Phase 2's corpus (which deliberately contains none of these).
- `layout::wrap`'s exact return shape — Section 4 says "`Vec<Line>` plus
  `heading_lines: Vec<usize>`" without specifying tuple vs. struct. →
  ASSUMPTION: a small `pub struct LayoutResult { lines: Vec<Line>,
  heading_lines: Vec<usize> }` for named-field clarity at call sites.
- H1/H2 underline width when the heading text itself must wrap at narrow
  widths ("the width of the text" is ambiguous if the text spans multiple
  wrapped lines) → ASSUMPTION: underline width = the heading's pre-wrap
  display width, capped at `content_width`. The corpus/snapshot test uses
  short headings so this edge case isn't exercised by acceptance criteria,
  but the implementation must not panic on it regardless.

---

## Tasks

### Implementation
- [x] 0. Create branch `feature/issue-4-phase2-rendering` from develop following docs/git-workflow.md
- [x] 1. `src/style.rs`: `Style` struct (`fg: Option<Color>`, `bold`, `italic`, `dim`, `strikethrough`, `reverse`, `underline` — `Clone + Copy + PartialEq + Default`) restricted to crossterm's 16 ANSI colors (no RGB, per Section 5); `Span { text: String, style: Style }`. Named palette constants for the colors this phase actually uses, so `render.rs`/`layout.rs` reference names instead of scattering raw `Color::X` literals: `pub const HEADING: Color = Color::Cyan;` (H1–H3), `pub const CODE: Color = Color::Yellow;` (inline code), `pub const LINK: Color = Color::Blue;` (link text). Dim-only usages (rule line, link url suffix, image placeholder) just set `dim: true` with `fg: None` — no constant needed. Alongside this task, update `docs/mdv-build-plan.md` Section 4's `Style` definition and `docs/architecture.md` to add the `underline` field (see Impact assessment). Unit tests: `Style::default()` is all-off/no-color.
- [x] 2. `src/render.rs` scaffolding: `Block` enum (`Heading { level: u8, spans: Vec<Span> }`, `Paragraph { spans: Vec<Span> }`, `Rule`), `Document { blocks: Vec<Block>, headings: Vec<TocEntry> }`, `TocEntry { level: u8, text: String, block_index: usize }`, `build_document(markdown: &str) -> Document` driven by a `pulldown_cmark::Parser` with `Options::ENABLE_TABLES | ENABLE_STRIKETHROUGH | ENABLE_TASKLISTS | ENABLE_FOOTNOTES`. Handle `Heading`/`Paragraph`/`Rule` events; any other event is skipped per the Open Questions assumption. Unit tests: markdown snippet → expected `Block` for a heading and a paragraph.
- [x] 3. `render.rs` inline formatting: within a block's span collection, handle `Strong`/`Emphasis`/`Strikethrough` (nestable via a small style-stack), inline `Code` (→ `style::CODE` colored span), and `Text`, combining styles correctly (e.g. bold+italic together). H1 uppercases **all** inline content (Section 5's "all inline content, including code spans" is deliberately broad — this covers link text, image alt text, and the dim url suffix too, not just plain text and code spans; don't special-case links/images as exempt). Unit tests: each element individually, plus one nested case (`**bold *italic* text**` — the corpus/snapshot's nesting example is capped at this two-marker depth; task 8's serializer doesn't need to handle deeper combinations).
- [x] 4. `render.rs` links, images, breaks: `Link` → span styled `style::LINK` + `underline: true`, followed by a Dim ` (url)` span — skip the url-suffix span when the link's rendered plain text is exactly equal to its `dest_url` (a plain string comparison; this is Section 5's own operational definition of "autolink," so don't branch on pulldown-cmark's `LinkType` instead). `Image` → single Dim `[image: alt]` span (alt text only — never fetch/render, per Section 1 non-goals). `SoftBreak` → a space (reflows with the paragraph). `HardBreak` → append a sentinel `Span { text: "\n".to_string(), style: Style::default() }` to the current block's spans; `layout.rs` (task 5) recognizes a span whose text is exactly the single character `"\n"` as a forced line break and flushes the line in progress without printing anything for it. This is unambiguous because pulldown-cmark never places a literal `\n` inside a `Text` event's content (source line breaks arrive as separate `SoftBreak`/`HardBreak` events), so `"\n"` can only appear via this deliberate marker. Unit tests: link with distinct url/text, autolink, image, hard break produces the sentinel, soft break produces a space.
- [x] 5. `src/layout.rs` sanitize + wrap core: move/adapt `main.rs`'s Phase 1 `sanitize()` (strip `\r`; tabs → single space — the Section 5 "4 spaces inside code blocks" branch is dead until Phase 3; C0/DEL/C1 → `�`) into `layout.rs`, applied to span text before wrapping. `wrap(&Document, terminal_width) -> LayoutResult`: `content_width = terminal_width.min(100)`. Treat each block's `Vec<Span>` as one continuous styled text stream — concatenate across span boundaries for word-break purposes, greedily pack words onto each output `Line` up to `content_width` using `unicode_width::UnicodeWidthStr` (never `str::len()`/`chars().count()`) for all width math. When a run must split mid-span to fit (at a word boundary, or at the width boundary for a single overlong word), emit the pieces as separate `Span`s on separate `Line`s, each carrying the original `Span`'s `Style` unchanged — this is the exact mechanism the Phase 2 accept criterion ("a bold word split across a wrap stays bold") depends on. The `"\n"`-sentinel `Span` (task 4) forces a line flush wherever it's encountered, contributing no visible text. `Rule` → one `Line` of Dim `─` repeated to `content_width`. Unit tests: wrap at exact width boundary, CJK (`你好`)/emoji width, style preserved across a wrap, hard break splits at the sentinel, 200-char unbroken word, the two Section 12 security tests (ESC-byte + OSC52 sanitization; no-panic at widths 1/2/40 rendering `corpus.md` plus a seeded-LCG 10 kB pseudo-random string — hand-rolled LCG, no `rand` dependency).
- [x] 6. `layout.rs` heading presentation and block spacing: H1 → Bold+`style::HEADING` uppercase + `═` underline (width per Open Questions assumption); H2 → Bold+`style::HEADING` + `─` underline; H3 → Bold+`style::HEADING`, no underline; H4–H6 → Bold, `§ ` prefix, no color. Block spacing (decided, uniform): insert exactly one blank `Line` between every adjacent pair of top-level blocks, in any combination (Heading-Heading, Heading-Paragraph, Paragraph-Rule, etc.) — never two consecutive blank lines even where two rules would otherwise both apply, and no leading/trailing blank line at the very start/end of the document. Populate `heading_lines` (first wrapped-line index per `TocEntry`). Unit tests: expected `Line`/style output per heading level, exactly one blank-line separator between every pairing of the three block kinds, no blank line before the first block or after the last.
- [x] 7. Wire `main.rs` + `view.rs`: add `mod style; mod render; mod layout;`. In `run()`, replace the sanitize+split codepath with `render::build_document(&config.contents)` → `layout::wrap(&document, width)` → `ViewState`. Retype `ViewState.lines` (`view.rs`) from `Vec<String>` to `Vec<Line>`; update `draw()` to print each `Span` by setting its full style explicitly (`SetForegroundColor`, `SetAttribute` for bold/dim/italic/strikethrough/underline/reverse) rather than incrementally diffing against the previous span, then issuing `SetAttribute(Attribute::Reset)` (and implicitly resetting color) immediately after each span's text — this guarantees style never bleeds onto the next span or line regardless of what preceded it. Handle `Event::Resize(w, _)`: re-run `layout::wrap` at the new width, clamp `ViewState`'s offset to the new `max_offset`, redraw. Update `view.rs`'s existing boundary tests for the `Vec<Line>` type.
- [x] 8. `tests/corpus.md` + snapshot test: corpus covering every Phase-2 element (H1–H6, paragraph, bold/italic/strikethrough incl. one nested combo, inline code, link with distinct url, autolink, image, horizontal rule, hard break, soft break, CJK text, emoji, one 200-char unbroken word). The corpus deliberately never combines inline code with bold/italic/strikethrough on the same run of text, so the serializer (below) never needs a code+other-marker composition rule. Snapshot test rendering the corpus at width 80 into plain text with re-serialized markers, diffed against checked-in `tests/snapshots/corpus.txt` (`UPDATE_SNAPSHOTS=1` regenerates it; no snapshot crate). **Serialization scheme** (fixed for this and all future phases' snapshot tests): for each `Span`, wrap its literal text with markers for attributes that have a natural plain-text convention, outermost-to-innermost: bold → `**...**`, italic → `*...*`, strikethrough → `~~...~~`, `style::CODE` foreground → `` `...` `` (mutually exclusive with the other three per the corpus constraint above). Dim/underline/link-color and heading-color are deliberately **not** given their own markers — they have no natural plain-text convention, and are instead verified precisely by the `render.rs`/`layout.rs` unit tests in tasks 3, 4, and 6 asserting directly on `Style` fields. The snapshot's job is layout/formatting-marker stability (text content, wrapping, blank-line placement, bold/italic/strike/code), not full attribute coverage.
- [x] 9. Manual acceptance verification against the Phase 2 accept criteria (Section 9): corpus renders with correct styling, resizing the terminal reflows paragraphs live, a bold word split across a wrap stays bold on both lines. Record the outcome in the PR description (pty-driven harness, matching Phase 1's approach).
      Verified via a pty + `pyte` terminal-emulation harness (same approach as Phase 1): spawned the real debug binary under a pseudo-TTY at 100x30, fed `tests/corpus.md`, and inspected the emulated screen buffer plus its bold/color cell attributes. Results: (1) all Phase 2 elements rendered as specified — heading colors/bold per level, `§ ` prefix on H4–H6, `═`/`─` underlines sized correctly, dim rule spanning the full content width, link text followed by a dim `(url)` suffix, autolink with no suffix, image alt placeholder, CJK/emoji intact; (2) resizing the pty from 100 to 40 columns and sending `SIGWINCH` triggered a live reflow — the same corpus re-wrapped correctly at the new width, headings/rules resized to the new content width; (3) a dedicated narrow-width (10 cols) test with a single 26-char bold word confirmed the word splits across 3 lines with every fragment carrying the bold attribute in the emulated screen buffer.
- [x] 10. **FIX REQUIRED (second adversarial-review pass, ref #4):** `render.rs`'s `image_alt: Option<String>` was a single slot, not a stack, so a `Link` nested inside an `Image`'s alt text (`![a [link](url) cat](cat.png)`) leaked a visible `" (url)"` span into the paragraph ahead of the image placeholder, and an `Image` nested inside another `Image`'s alt text (`![outer ![inner](a.png) text](b.png)`) corrupted state — the inner image's `End` push landed in `current_spans` while the outer was still "open," and the outer's trailing text then fell through to the non-alt branch and rendered as plain, undimmed text. Both are valid CommonMark that pulldown-cmark actually emits (confirmed empirically, not just by inspection). Fix: `image_alt` became a `Vec<String>` stack (one entry per open `Image`, pushed/popped on `Start`/`End`); `Text`/`Code`/`SoftBreak` route to `image_alt.last_mut()` when non-empty; nested `End(Image)` folds the inner alt text (unwrapped, no `[image: ]` brackets) into the new top rather than leaking a `Span`. `link_stack`'s start marker became an enum (`Spans(usize)` / `Alt(usize)`) recording which buffer the link's content lands in, so the autolink rendered-text comparison and the url-suffix push both target the correct buffer whether or not the link is nested inside an image. Added regression tests for both scenarios. Re-ran adversarial-review: second pass PASS (see Finish checklist).

### Finish
- [x] Write / update tests for all implementation tasks above
- [x] Run full test suite — all tests pass (66 passed)
- [x] Run `cargo audit` — confirm still clean (no new dependencies added this phase)
- [x] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
      First pass: one FIX REQUIRED finding (heading with no content producing an
      out-of-bounds/misattributed `heading_lines` index) — fixed; re-ran the
      skill, second pass PASS. One LOW finding (grapheme-cluster wrap
      splitting) deferred — see Deferred findings below.
      Third pass (new session, ref #4): one FIX REQUIRED finding (nested
      Image/Link alt-text state corruption) — fixed as task 10 above; two
      LOW findings (heading color override on dim spans, `TocEntry` text
      including link/image url suffixes) deferred — see Deferred findings
      below. Re-ran the skill, fourth pass PASS.
- [x] Update `docs/architecture.md` if actual module boundaries/data flow diverge from what's documented there (beyond the `Style.underline` fix already folded into task 1)
      No divergence — the existing module layout, data flow, and boundaries
      sections already describe exactly what Phase 2 built (they were
      written in anticipation of it during Phase 1).
- [x] Update `README.md` if affected — updated the status line from Phase 1
      to Phases 1–2.
- [x] Convert draft PR to ready-for-review; add `Closes #4` to PR description;
      set this plan's `_Status:_` to `READY`
- [x] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 4 --remove-label agent --remove-label "in progress" --add-label needs-review`

---

## Deferred findings
<!-- populated after adversarial-review, if any LOW findings are deferred -->

- **LOW — multi-codepoint grapheme clusters can split across a line-wrap
  boundary.** `layout::split_at_width` (and `place_piece`'s force-through
  path) operate on `char` boundaries using `unicode_width`'s per-codepoint
  widths, not extended grapheme clusters. A sequence made of multiple
  codepoints that render as one glyph (e.g. an emoji + skin-tone modifier, or
  a flag built from two regional-indicator codepoints) could theoretically be
  split between codepoints if a wrap boundary lands between them, rendering
  as two separate glyphs instead of one. This never panics (the split is
  always on a valid UTF-8/char boundary) — it's a rendering-fidelity gap, not
  a crash or security issue. Fixing it properly needs grapheme-cluster
  segmentation (e.g. the `unicode-segmentation` crate), which would add a 4th
  dependency beyond the 3 named in the build plan (Section 2's hard
  constraint) — out of scope for Phase 2. The build plan's own CJK/emoji
  requirement (Section 5/12) is stated in terms of `unicode_width` column
  math on simple single-codepoint characters, which the corpus and unit
  tests already cover; combining characters aren't in the corpus. Revisit
  only if a real document surfaces visibly broken glyphs.

- **LOW — a link's dim url-suffix or an image's dim alt placeholder gets
  promoted to bold + heading color when nested inside an H1–H3 heading.**
  `layout::heading_presentation_spans` only checks `style.fg.is_none()` to
  decide whether a span already "owns" its color; it doesn't also check
  `dim`. A dim-only span (`fg: None, dim: true`) therefore gets `fg:
  Some(HEADING)` layered on top, alongside its existing `dim: true` —  a
  bold+dim+cyan combination never intended by the spec, instead of staying
  visually neutral/dim. Never panics, not exercised by the corpus (which
  doesn't nest a link/image inside a heading). Low visual-fidelity impact;
  revisit if a real heading with an embedded link/image surfaces the
  combination looking wrong in practice.

- **LOW — `TocEntry.text` includes the auto-appended link/image url suffix.**
  `render.rs`'s `End(TagEnd::Heading)` builds `TocEntry.text` by
  concatenating every span's text, including the dim `" (url)"` suffix
  `End(TagEnd::Link)` appends and the `"[image: alt]"` placeholder
  `End(TagEnd::Image)` appends. A heading like `## [Link Text](url)` would
  produce a TOC entry of `"Link Text (url)"` rather than `"Link Text"`.
  Currently inert — `Document.headings` has no consumer until Phase 5 (see
  Out of scope) — so this doesn't affect Phase 2's acceptance criteria.
  Flagged now so Phase 5 doesn't rediscover it as a surprise; fix then by
  computing TOC text from only the non-suffix spans, or by tagging the
  suffix/placeholder spans so they can be excluded.
