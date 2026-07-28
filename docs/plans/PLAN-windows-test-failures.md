# PLAN: windows-test-failures

_Branch:_ `fix/issue-18-windows-test-failures`
_Date:_ 2026-07-28
_Status:_ IN PROGRESS
_Source:_ #18
_PR:_ #<pr-number>
<!-- filled in after first push; omit until then -->

---

## Problem

CI's new `windows-latest` matrix entry (#15/#17) surfaced 2 pre-existing test
failures in `src/main.rs`, both test-code portability gaps rather than real
`mdv` bugs:

1. `unreadable_file_is_an_error` (line ~760) hardcodes the Unix `io::Error`
   message text `"No such file or directory"`. Windows produces different
   message text for the same `ErrorKind::NotFound`, so the exact-string
   assertion fails on Windows even though the underlying behavior is correct.
2. `corpus_snapshot_at_width_80` (line ~832) compares freshly-rendered output
   (always `\n`-terminated, per `serialize_lines`) against the checked-in
   snapshot `tests/snapshots/corpus.txt` read from disk. On Windows runners,
   git's default `core.autocrlf` checks that file out with `\r\n` endings,
   so the comparison spuriously fails.

## Out of scope

- Adding a `.gitattributes` file or otherwise changing checkout/line-ending
  behavior at the repo level — the fix normalizes the comparison in the test
  itself instead, which is minimal and sufficient.
- Any change to production code paths (`clean_io_message`, `parse_args`,
  rendering/layout). Both failures are test-only; production error handling
  is already platform-agnostic.
- Adding CI Windows-specific skips or `#[cfg(not(windows))]` on either test —
  the goal is for both tests to pass unmodified in behavior on all platforms,
  not to exclude Windows from coverage.

## Impact assessment

- `src/main.rs` only, both changes confined to the `#[cfg(test)] mod tests`
  block (module boundaries in `docs/architecture.md` unaffected).
- No new dependencies, no `Cargo.toml` change.
- No risk to release/runtime behavior — test-only diff.

## Open questions

- None blocking. Assumption: for `unreadable_file_is_an_error`, rather than
  hardcoding a second platform-specific string behind `cfg(windows)`, the
  test computes its expected OS message dynamically by calling
  `fs::read(&path)` itself and reusing the crate's own `clean_io_message` on
  the resulting `io::Error` — this stays correct on any platform without
  encoding a second literal. [ASSUMPTION — proceeding on this basis]
- Why `corpus_snapshot_at_width_80` only normalizes the snapshot
  (`tests/snapshots/corpus.txt`) read, not the `tests/corpus.md` source read:
  normalizing the source before parsing would mean the test never actually
  exercises whatever CRLF-sourced content looks like once checked out on
  Windows — it would silently mask a real pulldown-cmark CRLF-vs-LF parsing
  difference if one exists, rather than testing real on-disk content. Task 3
  (CI verification) is the empirical check for whether this narrower scope is
  actually sufficient; if the test still fails on `windows-latest` after
  tasks 1–2, that is a new finding to capture, not something to
  work around by widening the normalization speculatively.

---

## Tasks

### Implementation
- [ ] 0. Create branch `fix/issue-18-windows-test-failures` from develop following docs/git-workflow.md
- [x] 1. Fix `unreadable_file_is_an_error`: instead of asserting against the
      hardcoded Unix string, call `fs::read(&path)` directly in the test to
      get a fresh `io::Error` for the same (still-missing) path, pass it
      through the crate's existing `clean_io_message`, and build the
      expected string with the same `format!("mdv: cannot read '{}': {}",
      path.display(), ...)` pattern production uses at `src/main.rs:105-111`.
      Assert equality against that dynamically-built string, not a literal.
- [ ] 2. Fix `corpus_snapshot_at_width_80`: normalize `\r\n` → `\n` on the
      snapshot file (`tests/snapshots/corpus.txt`) read from disk before
      comparing against the in-memory rendered string. Do not touch the
      `tests/corpus.md` read — see Open questions for why.
- [ ] 3. Push the branch and open/update the draft PR targeting `develop`,
      then confirm the GitHub Actions `test (windows-latest)` job passes
      with both `unreadable_file_is_an_error` and `corpus_snapshot_at_width_80`
      green. This is the only real signal for this issue — local test runs
      on this machine cannot reproduce the Windows-specific failures. If
      either test still fails on `windows-latest`, do not paper over it —
      capture what actually failed and add a follow-up task to this plan
      before continuing to Finish.

### Finish
- [ ] Write / update tests for all implementation tasks above
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected
- [ ] Convert draft PR to ready-for-review; add `Closes #18` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 18 --remove-label agent --remove-label "in progress" --add-label needs-review`

---
