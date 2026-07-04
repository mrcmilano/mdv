# PLAN: Phase 1 — Skeleton and terminal lifecycle

_Branch:_ `feature/issue-1-phase1-skeleton`
_Date:_ 2026-07-04
_Status:_ READY
_Source:_ #1
_PR:_ #<pr-number>
<!-- filled in after first push -->

---

## Problem

`mdv` is a brand-new crate — no `Cargo.toml`, no `src/` yet. This plan implements
Build Plan (`docs/mdv-build-plan.md`) **Section 9, Phase 1**: CLI parsing, file
loading, alternate-screen/raw-mode terminal lifecycle with a panic hook, and an
event loop that displays the raw file text with `j`/`k`/`g`/`G`/`q` scrolling —
the minimum needed to open a file, scroll it, and quit cleanly (including on
`Ctrl-C` or an injected panic). No Markdown rendering yet; that is Phase 2
(#4).

## Out of scope

- Markdown parsing/rendering (`pulldown-cmark` integration, `render.rs`,
  `layout.rs`, `style.rs`) — Phase 2 (#4).
- Line wrapping to `content_width` (Section 5) — the raw-line display in this
  phase prints file lines as-is; proper wrapping arrives with `layout.rs` in
  Phase 2.
- Status bar (Section 8) — its percentage/hint format references features
  (`t:toc`, `/:search`) that don't exist until later phases; deferred to
  Phase 2 alongside the real layout pipeline.
- Search, TOC, tables, task lists, footnotes, syntax highlighting, `--watch`,
  stdin input, OSC 8 hyperlinks — later phases / stretch goals, untouched.
- `d`/`u`/`PageDown`/`PageUp`/`Space` half-page scroll is IN scope (see Open
  questions) even though the Phase 1 acceptance line only names `j/k/g/G/q`.

## Impact assessment

Greenfield crate — no existing code is affected.

New files: `Cargo.toml`, `Cargo.lock` (committed per Section 12),
`src/main.rs`, `src/input.rs`, `src/view.rs`. `src/render.rs`, `src/layout.rs`,
`src/style.rs` are **not** created yet (see Open questions) — they land in
Phase 2 when they have real content.

Risk: incorrect raw-mode/alternate-screen teardown can leave the user's real
terminal in a broken state (no cursor, raw mode stuck on) if the process exits
through an unhandled path. Mitigated by the panic hook (task 6) and an
explicit manual verification task (task 8) covering normal quit, `Ctrl-C`,
and an injected `panic!()`.

`cargo audit` must run after the first successful build (Security policy) —
included in Finish.

## Open questions

- Should Phase 1 apply the Section 5 sanitization rules (strip `\r`, expand
  tabs, replace control/DEL/C1 bytes with `�`) to the raw file lines before
  printing, even though the formal `Span`/`Document` pipeline doesn't exist
  until Phase 2? → ASSUMPTION: yes — Section 12's escape-injection invariant
  ("no exception for code blocks or raw HTML") is unconditional, and Phase 1
  already writes untrusted file bytes straight to the terminal, so it applies
  here too. Implemented as a small shared sanitize function reused by
  `layout.rs` in Phase 2.
- Should Phase 1 implement the full Normal-mode scroll keybinding row from
  Section 7 (`j/k/↓/↑`, `d/PageDown/Space`, `u/PageUp`, `g/Home`, `G/End`,
  `q/Ctrl-C`), or only the literal `j/k/g/G/q` named in the Phase 1 accept
  line? → ASSUMPTION: implement the full scroll row now (excluding `t`, `/`,
  `n`, `N`, which depend on state that doesn't exist until Phase 5) — the
  keybinding table is already fully specified and these are trivial aliases,
  so building them now avoids rework.
- Should `Cargo.toml` declare all 3 dependencies (`pulldown-cmark`,
  `crossterm`, `unicode-width`) now, even though Phase 1 code only uses
  `crossterm`? → ASSUMPTION: yes — Section 2 defines the dependency set as
  one-time project setup, and `cargo audit` (run at the end of this phase)
  should cover the full dependency set from the start.
- Should Phase 1's event loop handle `Event::Resize` at all, given the
  build plan's general terminal-lifecycle spec ("re-run layout, clamp scroll
  offset, redraw") depends on `layout.rs`, which doesn't exist until Phase 2?
  → DECIDED (user, 2026-07-04): ignore `Event::Resize` entirely in Phase 1.
  `ViewState`'s `viewport_height` is fixed at construction; proper resize
  handling arrives with `layout.rs` in Phase 2.

---

## Tasks

### Implementation
- [x] 0. Create branch `feature/issue-1-phase1-skeleton` from develop following docs/git-workflow.md
- [x] 1. Crate scaffolding: `Cargo.toml` with the 3 dependencies and release profile from Section 2, minimal `src/main.rs` with `#![forbid(unsafe_code)]` and an empty `fn main()`. `cargo build` and `cargo clippy -- -D warnings` clean.
- [x] 2. CLI parsing & validation: positional `<FILE>`, `--help`/`-h`, `--version`/`-V`; error cases (no arg, unknown flag, unreadable file, non-UTF-8 file) print the one-line stderr message from Section 3 and exit 1; `--help` prints usage + the Section 7 keybinding table to stdout and exits 0; `--version` prints `mdv <version>` via `CARGO_PKG_VERSION` and exits 0. Parsing logic in a testable function with unit tests for each case.
- [x] 3. `src/input.rs`: `KeyEvent -> Action` mapping for the Phase 1 scroll row decided in Open questions (line up/down, half-page up/down, top, bottom, quit). Unit tests covering every bound key and confirming unbound keys map to no-op.
- [x] 4. `src/view.rs`: minimal `ViewState` (scroll `offset`, raw lines, viewport height) with clamped scroll arithmetic (`max_offset = lines.len().saturating_sub(viewport_height)`, never negative). Unit tests for boundary conditions: empty file, single line, file shorter than viewport, scrolling past top/bottom.
- [x] 5. Sanitization helper per Section 5 (strip `\r`; tabs → single space outside code blocks — no code-block concept yet in Phase 1, so tabs → single space unconditionally; other C0/DEL/C1 bytes → `�`), applied to file lines before they enter `ViewState`. Unit test: a line containing a raw ESC byte (`0x1b`) followed by an OSC 52 sequence — assert no `�`-replaced line contains byte `0x1b`.
- [x] 6. Terminal lifecycle in `main.rs`: enter alternate screen + raw mode + hide cursor on start; panic hook that restores the terminal before printing the panic message, then chains the previous hook; restore on every exit path. Wire the stdout-not-a-TTY precondition check (Section 3) to run before entering raw mode.
- [x] 7. Event loop wiring in `main.rs`: read file → sanitize (task 5) → split into raw lines → build `ViewState` (task 4) → blocking `crossterm::event::read()` → map via `input::map` (task 3) → mutate state → redraw only on change, printing just the visible line slice.
- [x] 8. Manual acceptance verification against the Phase 1 accept criteria: open a file, scroll with the full key row, quit with `q` and with `Ctrl-C`, confirm an injected `panic!()` still leaves the terminal sane, confirm piping stdout errors out per Section 3. Record the outcome in the PR description.
      Verified via a pty-driven harness (this shell has no controlling terminal, so a real interactive session isn't directly reachable): spawned the real binary under a pseudo-TTY, sent raw key bytes, and inspected the output stream. Results: (1) scroll `j`/`G`/`g` moved the visible window correctly; (2) `q` and `Ctrl-C` both exited cleanly (code 0) with `LeaveAlternateScreen`/cursor-`Show` sequences present; (3) a temporarily-injected `panic!()` (behind an env var, reverted after the test — not part of the committed code) exited with code 101 and the terminal-restore escape sequences appeared in the output *before* the panic message, confirming the panic hook ordering; (4) piping stdout exits 1 with the exact Section 3 message.

### Finish
- [x] Write / update tests for all implementation tasks above
- [x] Run full test suite — all tests pass (35 tests, `cargo test`)
- [x] Run `cargo audit` (first successful build) — no advisories
- [x] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
      2 FIX REQUIRED findings resolved (terminal-state leak on partial init
      failure; full-viewport scroll bug in `draw()` — both verified fixed via
      a pty + `pyte` terminal-emulation harness). 1 LOW finding deferred, see
      below. Second pass: PASS.
- [ ] Update `README.md` if affected — N/A, no `README.md` exists yet in this repo.
- [x] Convert draft PR to ready-for-review; add `Closes #1` to PR description;
      set this plan's `_Status:_` to `READY`
- [x] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 1 --remove-label agent --remove-label "in progress" --add-label needs-review`

---

## Deferred findings
<!-- populated after adversarial-review, if any LOW findings are deferred -->

- **LOW — no `--` end-of-options marker in `parse_args`** (`src/main.rs`):
  a file literally named `-foo.md` can never be opened; it is always parsed
  as an unknown flag (`mdv: unknown option '-foo.md'`). Build plan Section 3
  doesn't mention `--` handling or dash-prefixed filenames anywhere, and this
  is a narrow edge case unlikely to matter in practice. Deferring rather than
  adding unspecified CLI behavior.
