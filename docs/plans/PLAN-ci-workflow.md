# PLAN: CI workflow (fmt/clippy/test/audit)

_Branch:_ `chore/issue-15-ci-workflow`
_Date:_ 2026-07-28
_Status:_ READY
_Source:_ #15
_PR:_ #<pr-number>
<!-- filled in after first push; omit until then -->

---

## Problem

There is no `.github/workflows/` in the repo. Every gate described in
AGENTS.md (`cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test`,
`cargo audit`) is enforced only by convention — nothing runs it automatically
on push or PR. Add GitHub Actions workflows that run the fmt/clippy/test loop
on every push/PR, and `cargo audit` when `Cargo.lock` changes.

## Out of scope

- Required status checks / branch protection. This repo is private and on
  the GitHub Free plan; both classic branch-protection rules and the newer
  rulesets return `403: Upgrade to GitHub Pro or make this repository
  public` (confirmed against the live repo during Assess). The workflows
  will run and report pass/fail on every push/PR, but nothing will
  technically block a merge yet. Revisit if the repo goes Pro or public —
  tracked as a follow-up, not blocking this issue.
- Scheduled (cron) `cargo audit` runs — decided against; Cargo.lock-change
  trigger only, per issue's proposed scope and user preference.
- Build caching (e.g. `Swatinem/rust-cache`) — dependency tree is small
  (3 direct deps), cold builds are already fast; skipped to keep the CI
  trust surface minimal.
- MSRV matrix / multiple Rust channels — only `stable` is tested, matching
  what AGENTS.md's commands already assume. No MSRV is documented anywhere
  in the build plan.
- Any change to `Cargo.toml` dependencies or runtime behavior — this is CI
  config only.

## Impact assessment

New files only, no source changes:
- `.github/workflows/ci.yml` — three separate jobs, all sharing the same
  triggers (`push` to `[main, develop]`; `pull_request` targeting
  `[main, develop]`):
  - `fmt` (ubuntu-latest only, output is OS-independent):
    `dtolnay/rust-toolchain@stable` with the `rustfmt` component; runs
    `cargo fmt --check`.
  - `clippy` (matrix: ubuntu-latest, macos-latest, windows-latest —
    crossterm has real per-OS code paths worth exercising):
    `dtolnay/rust-toolchain@stable` with the `clippy` component; runs
    `cargo clippy --locked -- -D warnings`.
  - `test` (same matrix): `dtolnay/rust-toolchain@stable`, no extra
    components; runs `cargo test --locked`.
  - Kept as three separate jobs rather than combined per-OS steps: each
    check reports independently in the PR checks list (no ambiguity about
    which failed), and clippy/test run in parallel instead of sequentially
    — free, since there's no shared cache to lose either way.
  - `--locked` is scoped to `clippy` and `test` only. Verified empirically
    that `cargo fmt --locked --check` fails
    (`error: unexpected argument '--locked' found`) — `cargo fmt` has no
    such flag, so it must not be added to the `fmt` job.
- `.github/workflows/audit.yml` — single `ubuntu-latest` job, same triggers
  as above. Detects whether `Cargo.lock` changed via a plain
  `git diff --name-only` step against the previous/base commit (no
  third-party diff-detection Action — keeps the Action count minimal,
  consistent with the no-caching decision), then runs `rustsec/audit-check`
  only when it did, gated by a step-level `if:` — not a trigger-level
  `paths:` filter and not a job-level `if:`. This avoids the known GitHub
  Actions gotcha where a required check gated at the trigger or job level
  never reports and can hang a PR forever. Documented here since it matters
  if required checks are ever turned on later.
- `README.md` — add CI status badges (ci.yml, audit.yml) near the top.

No cargo dependency changes (AGENTS.md's 3-dependency limit is untouched —
these are GitHub Actions, not crates). No runtime/render code touched.

Third-party Actions trust surface added: `actions/checkout`,
`dtolnay/rust-toolchain`, `rustsec/audit-check` — all pinned to
major-version tags (e.g. `@v4`), not exact commit SHAs. This is the
standard, low-maintenance default; it trusts each action's author not to
move the tag maliciously. Stricter SHA-pinning was considered and declined
— it would require `.github/dependabot.yml` (github-actions ecosystem) to
avoid pins silently going stale, which is scope beyond issue #15.

## Resolved decisions (from Assess discussion)

- OS matrix: Ubuntu + macOS + Windows for clippy/test (separate jobs, not
  combined).
- `cargo audit` trigger: on push/PR when `Cargo.lock` changes (not
  scheduled), detected via plain `git diff`, not a third-party Action.
- No build caching.
- GitHub Actions pinned to major-version tags, not commit SHAs.
- Branch protection / required checks: deferred, see Out of scope.

---

## Tasks

### Implementation
- [x] 0. Create branch `chore/issue-15-ci-workflow` from develop following docs/git-workflow.md
- [x] 1. Add `.github/workflows/ci.yml` with three separate jobs — `fmt` (ubuntu-latest, `rustfmt` component), `clippy` (matrix: ubuntu/macos/windows, `clippy` component, `--locked`), `test` (same matrix, no extra components, `--locked`) — triggered on push/PR to `main`/`develop`
- [x] 2. Add `.github/workflows/audit.yml`: single ubuntu-latest job, same triggers, using a plain `git diff --name-only` step to detect `Cargo.lock` changes and a step-level `if:` to gate `rustsec/audit-check`
- [x] 3. Add CI status badges for both workflows to `README.md`

### Finish
- [x] Write / update tests for all implementation tasks above
      (N/A for CI-only config — verification is the workflows running green
      on the draft PR itself; note this explicitly rather than skipping
      silently. `fmt`/`clippy`/`test` green on ubuntu-latest and macos-latest,
      and `audit` green on ubuntu-latest; `test (windows-latest)` red due to
      pre-existing issues, see Deferred findings and #18)
- [x] Run full test suite — all tests pass locally (167/167); CI matrix
      confirms green on ubuntu/macos, see above for the one known exception
- [x] Run `/skill:adversarial-review` — PASS, no FIX REQUIRED findings; one
      LOW finding documented in Deferred findings below
- [x] Update `README.md` if affected — CI/Audit badges added (task 3)
- [x] Convert draft PR to ready-for-review; add `Closes #15` to PR description;
      set this plan's `_Status:_` to `READY`
- [x] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 15 --remove-label agent --remove-label "in progress" --add-label needs-review`

---

## Deferred findings
<!-- Populated only if adversarial-review produces LOW findings that are deferred. -->

- **`test (windows-latest)` fails on this PR's own CI run — 2 pre-existing test
  failures, not caused by this change.** The new OS matrix (a deliberate
  plan decision, see "Resolved decisions") surfaced two portability gaps in
  the existing test suite that no prior CI ever exercised:
  - `unreadable_file_is_an_error` (`src/main.rs:764`) asserts the exact Unix
    OS error string `"No such file or directory"`, which differs on Windows.
  - `corpus_snapshot_at_width_80` (`src/main.rs:847`) compares in-memory
    `\n`-terminated output against a snapshot file that Windows runners
    check out with `\r\n` line endings (`core.autocrlf`).
  - Rationale for deferring: fixing these requires source-file changes,
    which contradicts this plan's stated scope ("New files only, no source
    changes" — Impact assessment). User decision (Assess-equivalent
    discussion during Finish): leave the CI config as specified, document
    here, and track the source fix separately.
  - Tracked as follow-up: #18. The `windows-latest` `test` job will show red
    on this PR and will continue to do so until #18 is resolved — this is
    expected and not a defect in the CI workflow itself.
