# PLAN: sanitize-filename

_Branch:_ `fix/issue-21-sanitize-filename`
_Date:_ 2026-08-03
_Status:_ IN PROGRESS
_Source:_ #21

---

## Problem

`RunConfig.filename` (`src/main.rs`) is derived from `path.file_name()` in
`parse_args` and printed raw into the status bar every redraw (`draw()` via
`Print(&line)`, `status_bar_line()`). Unlike the Markdown file's *content*,
which always passes through `layout::sanitize`/`sanitize_code` before
reaching the terminal (build plan Section 12: "no exception for code blocks
or raw HTML"), the filename never does. On Unix a filename may contain any
byte except `/` and NUL, including ESC (`0x1B`), so a maliciously-named file
(e.g. embedding an OSC 52 clipboard-write sequence) can inject raw terminal
escape sequences on every frame just by being opened with `mdv`, with no
involvement of the file's content. This closes that gap by routing
`filename` through the existing sanitizer at load time.

## Out of scope

- Any change to how Markdown *content* is sanitized — that path is already
  correct and untouched.
- Sanitizing the path used for `fs::read` / error messages (`path.display()`
  in the "cannot read" error) — that string goes to stderr before the
  terminal is ever put into raw/alternate-screen mode, not into the redrawn
  status bar, so it is a different (already-understood) exposure not covered
  by this issue.
- Truncation/width handling of the status bar — unaffected, `sanitize`
  preserves character count except for `\r`/tab/control substitutions.
- Changing `layout::sanitize`'s general `\n`-preserving behavior. That
  behavior is correct and required for Markdown body content (line breaks
  are meaningful there) and must not change. The filename path needs one
  extra, filename-specific step on top of it (see task 2) — not a change to
  the shared sanitizer.

## Impact assessment

- `src/layout.rs`: change `sanitize` from private (`fn`) to crate-visible
  (`pub(crate) fn`) so `main.rs` can call it. No behavior change to the
  function itself.
- `src/main.rs`: new private helper `sanitize_filename` (see task 2), and one
  call site in `parse_args` (line ~116-120) that uses it instead of the
  direct `.to_string()`. Everything downstream (`status_bar_line`, `draw`) is
  already correct once the input is clean, no other call sites need changes.
- No new dependencies, no `Cargo.toml` change, no module-boundary change
  (`layout` already owns sanitization per `docs/architecture.md`).

## Open questions

None blocking.

## Note on scope beyond the issue text

The issue's "Suggested direction" says to reuse `layout::sanitize` and test
via `RunConfig`/`status_bar_line`. Two adjustments were made to that
suggestion after reviewing the actual code, both reflected in task 2 below:

1. `layout::sanitize` alone is not sufficient: it intentionally lets `\n`
   through (needed for Markdown body text), but a filename is rendered as a
   single terminal line, so a raw newline in a filename would still reach
   the terminal unsanitized after calling `sanitize` verbatim. The fix needs
   one additional, filename-specific step.
2. The regression test must exercise `parse_args` (where the fix actually
   lives), not `RunConfig`/`status_bar_line` (which never sanitize anything
   and would pass regardless of whether the fix is wired up).

---

## Tasks

### Implementation
- [x] 0. Create branch `fix/issue-21-sanitize-filename` from develop following docs/git-workflow.md
- [x] 1. In `src/layout.rs`, change `fn sanitize(text: &str) -> String` to
      `pub(crate) fn sanitize(...)` (leave `sanitize_with_tab`/`sanitize_code`
      private — only `sanitize` is needed outside the module).
- [x] 2. In `src/main.rs`, add a private helper
      `fn sanitize_filename(raw: &str) -> String` that calls
      `layout::sanitize(raw)` and additionally replaces any remaining `'\n'`
      with a single space (see "Note on scope beyond the issue text" above
      for why `sanitize` alone isn't enough here). In `parse_args`, replace
      the direct `.to_string()` on the computed filename with
      `sanitize_filename(...)`.

      Tests:
      - Unit tests directly on `sanitize_filename` (no disk I/O needed):
        an ESC byte is removed, an OSC 52 sequence
        (`"\x1b]52;c;<base64>\x07"`) is neutralized, and an embedded `\n` is
        replaced with a space. Analogous in spirit to
        `sanitization_neutralizes_esc_byte_and_osc52_sequence` in
        `layout.rs`, but exercised at the `main.rs` helper directly.
      - One `#[cfg(unix)]` integration test on `parse_args`: extend the
        existing `TempFile` test helper (src/main.rs:681) with a constructor
        that accepts an explicit raw-byte filename via
        `std::os::unix::ffi::OsStringExt` (mirroring how
        `non_utf8_argument_is_an_error_not_a_panic`, line ~704, already
        builds a raw-byte `OsString` on Unix), create a temp file whose name
        contains a raw ESC byte, call `parse_args` on its path, and assert
        the resulting `RunConfig.filename` contains no `0x1B` byte. This is
        the one test confirming `sanitize_filename` is actually wired into
        `parse_args`, not just correct in isolation. `#[cfg(unix)]`-gated
        because Windows' filesystem APIs reject control characters in
        filenames outright, so the same on-disk scenario can't be
        constructed there — same platform gap already documented in
        `docs/plans/PLAN-windows-test-failures.md` (issue #18).

### Finish
- [ ] Write / update tests for all implementation tasks above
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected
- [ ] Convert draft PR to ready-for-review; add `Closes #21` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 21 --remove-label agent --remove-label "in progress" --add-label needs-review`

---

## Deferred findings
<!-- populated only if adversarial-review produces deferred LOW findings -->
