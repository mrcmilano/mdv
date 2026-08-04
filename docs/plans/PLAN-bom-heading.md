# PLAN: bom-heading

_Branch:_ `fix/issue-22-bom-heading`
_Date:_ 2026-08-04
_Status:_ APPROVED
<!-- DRAFT        → APPROVED:      user approves via label flip (needs-review → agent, see AGENTS.md §5) -->
<!-- APPROVED     → IN PROGRESS:   first code task commit (task 1+); -->
<!--                              branch creation (task 0) does NOT flip it -->
<!-- IN PROGRESS  → READY:         PR converted to ready-for-review -->
<!-- READY is the terminal status this file records. Actual completion -->
<!-- ("done") only occurs when the PR is merged, by which point the plan -->
<!-- file is no longer being updated. -->
<!-- Each transition: update this field and commit the plan file. -->
_Source:_ #22
_PR:_ #<pr-number>
<!-- filled in after first push; omit until then                 -->

---

## Problem

A Markdown file that starts with a UTF-8 byte-order mark (`U+FEFF`) — a common
artifact of files saved by Windows editors — silently loses its first heading.
`pulldown-cmark` requires `#` to be the first character on the line (after at
most 3 spaces); a leading BOM is neither whitespace nor `#`, so the line parses
as a plain paragraph whose text begins with the invisible BOM character. The
heading loses its H1 styling and disappears from the TOC, with no error or
status-bar indication. `layout::sanitize` doesn't catch this because it runs
*after* `render::build_document` has already parsed the string, and `U+FEFF`
is outside the control-character ranges it strips anyway.

## Out of scope

- BOMs appearing mid-document (not as the very first character) — a rarer,
  separate case per the issue, needing its own decision on whether to render
  as `\u{FFFD}` or leave alone. Not addressed here.
- Any status-bar or other user-visible notice that a BOM was stripped. No
  existing precedent for this in the codebase (the filename-sanitization fix
  in #21/#23 strips silently too), so this fix follows the same pattern.
- UTF-16/UTF-32 BOMs — `main.rs` already rejects non-UTF-8 input via
  `String::from_utf8`, so these never reach this code path.

## Impact assessment

- `src/render.rs` — `build_document` gains a pre-parse step stripping a single
  leading `U+FEFF` before constructing the `pulldown_cmark::Parser`. This is
  the correct boundary per the issue: the bug is pre-parse, not a rendering
  concern, so it does not belong in `layout::sanitize`.
- No other module changes. `main.rs` needs no changes since `build_document`
  is the single call site that receives file contents (src/main.rs:524) as
  well as the only parsing entry point used by tests.
- Confirmed `config.contents` (the raw file string) is not retained or used
  for byte-offset purposes anywhere else — it is passed to `build_document`
  once and never referenced again (search and layout operate on the parsed
  `Document`'s `Span` text, not raw file offsets). So stripping a leading
  character before parsing cannot desync any offset-based logic elsewhere.
- Low risk: the change is additive (strips one specific leading character)
  and cannot affect documents that don't start with a BOM.

## Open questions

None — the issue's suggested direction is unambiguous and matches the
existing architecture (parsing boundary lives in `render.rs`; sanitization
in `layout.rs` is out of scope by design since it runs post-parse).

---

## Tasks

### Implementation
- [x] 0. Create branch `fix/issue-22-bom-heading` from develop following docs/git-workflow.md
- [ ] 1. In `render::build_document` (src/render.rs), strip a single leading
      `U+FEFF` from `markdown` before constructing `Parser::new_ext`, using
      `str::strip_prefix('\u{FEFF}')` (only strips if it's the very first
      character; no-op otherwise).
- [ ] 2. Add a unit test in `src/render.rs`'s test module asserting
      `build_document("\u{FEFF}# Hello")` produces a `Heading` block (level 1)
      — not a `Paragraph` — and a populated `TocEntry`. Note: level-1 headings
      are uppercased by existing logic (see
      `h1_uppercases_all_inline_content_including_code`, src/render.rs:939),
      so the expected span text and `TocEntry.text` are both `"HELLO"`, not
      `"Hello"` — mirror that test's assertion shape, not the level-2
      `heading_produces_heading_block_and_toc_entry` example, to avoid a
      test that fails against otherwise-correct output.
- [ ] 3. Add a second unit test locking in the "leading-only" scope boundary
      for a BOM appearing mid-document (not as the file's first character) —
      per the issue, this is explicitly out of scope and must not be
      stripped. Use a two-*paragraph* input with a blank-line separator
      (required by CommonMark for two distinct `Paragraph` blocks — adjacent
      lines with no blank line between them are one paragraph via lazy
      continuation), e.g. `"para\n\n\u{FEFF}# Heading"`, and assert
      `doc.blocks` equals two `Paragraph` blocks, where the second
      paragraph's span text is the literal, unstripped `"\u{FEFF}# Heading"`
      (i.e. no `Heading` block is produced and `doc.headings` is empty). A
      same-line case such as `"a\u{FEFF}# Hello"` is not a sufficient
      substitute here since it never has heading-parse potential in the
      first place ("#" isn't at the start of the line either way) and so
      wouldn't distinguish "leading-only" from "any line-start" stripping.

### Finish
- [ ] Write / update tests for all implementation tasks above
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected
- [ ] Convert draft PR to ready-for-review; add `Closes #22` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 22 --remove-label agent --remove-label "in progress" --add-label needs-review`

---
