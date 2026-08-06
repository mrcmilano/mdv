# PLAN: public release readiness

_Branch:_ `chore/issue-20-public-release-readiness`
_Date:_ 2026-08-05
_Status:_ IN PROGRESS
_Source:_ #20
_PR:_ #31

---

## Problem

The repo is ready to go public on quality grounds — tests, clippy, fmt and audit
are green, there is no `unsafe`, and a full `docs/` sweep found no secrets or
PII — but four things still stand in the way, and none of them are code defects.

`main` holds only the `init` commit: no `src/`, no `README.md`, no CI. Everything
real lives on `develop`, so a visitor landing on the default branch of a public
repo would see three files and conclude the project doesn't exist. CI's clippy
job lints only lib/bin targets, which is why two `bool_assert_comparison`
warnings have sat in `src/layout.rs` since Phase 3 without CI noticing.
`docs/api-error-handling.md` is 230 lines of generic frontend/API template
content that `AGENTS.md` itself declares inapplicable — visible scaffolding.
And `AGENTS.md` addresses two audiences in one unmarked file: universal
conventions any contributor needs, and the maintainer's plan-file/label ritual
that nobody else is expected to follow.

This plan clears all four, then promotes `develop` to `main` so the default
branch is the project. The visibility flip itself stays a human action.

## Out of scope

- **The visibility flip.** Human-only, and deliberately so — it is the one
  irreversible-in-practice step here. Documented in the Release section as a
  maintainer action with the exact command, never executed by the agent.
- **Everything in #30** — branch protection on `main`, repo topics, GitHub
  Discussions. Split out of #20 for a concrete reason: branch protection is not
  even configurable until the repo is public (the API returns
  `403 Upgrade to GitHub Pro or make this repository public`), so it cannot
  gate the flip that unlocks it.
- **Splitting `AGENTS.md` into two files.** Assessed in conversation and
  **rejected** — see #20's Decisions and the Open questions section below. Task 4
  implements the marking-in-place alternative. Do not re-derive the split.
- **Rewriting or squashing git history** before going public. Rejected in #20:
  it buys no real privacy and looks worse if noticed than being upfront.
- **The personal email in historical commits.** A conscious call already made
  and recorded in #20 (repo-local `user.email` is set to the GitHub noreply
  address for *future* commits). Going public makes it permanently visible; that
  was known when the call was made. Not reopened here.
- **`docs/session-retrospective-phase3-phase4.html`.** Untracked by deliberate
  choice per #20. It must **stay** untracked — task 5's verification includes
  confirming it did not get swept into a commit.
- **Any change to `src/` beyond the two assert rewrites in task 1.** No
  behaviour changes in this plan.
- **Changing what the workflow rules *say*.** Task 4 adds framing around
  `AGENTS.md` §1–§5; it does not edit, renumber, reorder or reword a single rule
  inside them. Zero impact on the maintainer's workflow is the explicit
  requirement, not a nice-to-have.

## Impact assessment

**Files changed**

| File | Change |
|---|---|
| `src/layout.rs` | 2 lines: `assert_eq!(…, true)` → `assert!(…)` (test module only) |
| `.github/workflows/ci.yml` | clippy job gains `--all-targets` (1 line) |
| `AGENTS.md` | Clippy command at **3** sites (L26, L47, L269). Audience banner + `## Workflow` divider. `docs/api-error-handling.md` reference removed (L284–285). |
| `CONTRIBUTING.md` | PR-bar clippy command (L19) |
| `README.md` | Development block clippy command (L90) |
| `docs/mdv-build-plan.md` | Phase acceptance-criteria clippy command (L311) |
| `docs/api-error-handling.md` | Deleted (230 lines) |
| `docs/plans/PLAN-public-release-readiness.md` | This plan |

**Verified on the ground before writing this plan** (dates matter — re-check at
implementation time if this sits unworked):

- `git merge-base --is-ancestor main develop` → true. `main` is **13 commits
  behind `develop` with zero divergence**, so promotion is a fast-forward. No
  conflicts, no merge strategy decision, nothing to resolve.
- `cargo clippy --all-targets --locked` produces **exactly 2 warnings**, both
  `bool_assert_comparison`, both in `src/layout.rs` (1048, 1066), both in the
  bin's test target. Nothing else surfaces under `--all-targets`.
- `tests/` contains only `corpus.md` and `snapshots/` — **no `.rs` integration
  test targets**. So `--all-targets` adds exactly one target (the bin's unit
  tests) over what CI lints today. This is a narrow, well-understood widening,
  not an open-ended one.
- `docs/api-error-handling.md` is referenced from **exactly one** place in the
  repo: the first bullet of `AGENTS.md`'s `## Domain Specific Rules`
  (`AGENTS.md:284–285`). Nothing else links it. Confirmed by grep across `*.md`,
  `*.yml`, `*.rs`.
- No branch protection exists on `main` (403, free private repo), so the release
  PR will merge without an approval gate.
- **Squash merge is enabled on this repo** (`squashMergeAllowed: true`, as are
  merge-commit and rebase). That makes "Squash and merge" a one-click way to
  collapse the entire project history into a single commit on `main` — exactly
  the history curation #20 rejected. R1 pins the merge method for this reason.
- `develop` is checked out in the **primary worktree** (`/Users/marco/Build/mdv`).
  Two other worktrees exist on unrelated branches, so task 0 must run from the
  primary worktree; a plain `git checkout -b … develop` works there.
- The clippy command is documented at **6 sites** outside `docs/plans/` — three
  of them in `AGENTS.md` alone, and one in `docs/mdv-build-plan.md`. Full
  enumeration with line numbers and context in task 3.

**Risks**

- **Clippy `--all-targets` may surface new warnings on other platforms.** The
  local check is macOS. The CI matrix runs ubuntu/macos/windows, and
  `src/main.rs` has `#[cfg]`-gated Windows paths that the local run never
  compiles. Task 2 therefore treats the CI run on the draft PR as the real
  verification, not the local run — see the task note.
- **The clippy command is documented in six places outside CI.** Changing CI
  without them leaves contributors — and the maintainer — passing locally and
  failing CI, which is the exact failure `CONTRIBUTING.md` already warns about
  for `Cargo.lock`. Task 3 exists to keep them in sync and is not optional
  polish.
- **Task 3 does change one thing the maintainer runs, deliberately and with the
  maintainer's explicit approval** — see the Open questions entry on
  `AGENTS.md:47`. The trivial-change gate becomes
  `cargo clippy --all-targets -- -D warnings`, so a one-line code fix committed
  outside the full workflow is linted with test code included. It is
  **unrelated to** the "zero workflow impact" requirement, which governs task 4
  (the `AGENTS.md` audience marking) only.
- **Deleting a doc is the one irreversible-feeling step in the develop-side
  work.** It isn't — it's a git-tracked file and the deletion is a revertible
  commit — but task 5 still confirms the single inbound reference is gone in the
  same commit, so no state exists where `AGENTS.md` points at a deleted file.

**Explicitly no impact on**

- The maintainer's workflow. Task 4 adds framing text only; §1–§5 keep their
  numbering, their order and their wording, and stay in the same file that
  `CLAUDE.md` already imports. Nothing about how a session loads or follows them
  changes.
- Test behaviour. The two assert rewrites are semantically identical
  (`assert_eq!(x, true)` ≡ `assert!(x)`); the test count must be unchanged
  before and after — verified in task 1.

## Open questions

All resolved in conversation before this plan was written. Recorded here for the
implementation record, because two of them reverse or close decisions written
into #20 earlier.

- **`main` branch strategy** → Keep `main` as the default branch; promote
  `develop` → `main` via a release PR. Closes the open question posted as a
  comment on #20. Rationale in #20's Decisions: the promotion is a clean
  fast-forward, and the alternative (default → `develop`, delete `main`) would
  need edits to `CONTRIBUTING.md`, `docs/git-workflow.md`, `AGENTS.md` and both
  CI branch filters to remove a step that runs rarely.
- **`AGENTS.md` split** → **Reversed.** #20 originally decided to split the file
  into a universal layer and a maintainer layer. The maintainer's decision is to
  **keep one file and mark two sections**, with the explicit requirement that the
  change be minimal and have zero impact on the current workflow. The accepted
  risk is stated in the maintainer's own words: an external contributor
  comfortable enough with AI agents to be tripped up by an unmarked ritual will
  bring their own workflow and overwrite `AGENTS.md` anyway. #20's body has been
  updated to record the reversal and its reasoning.
- **Post-public repo hygiene** → Split out to **#30**, not folded in here.
- **`docs/api-error-handling.md`** → Delete, not keep-and-caveat. It documents
  behaviour this project does not have; a caveat would leave 230 lines of
  inapplicable content in a public repo to no benefit.
- **Widening `AGENTS.md:47`'s trivial-change gate** (the one site in task 3 that
  changes what the maintainer actually runs, rather than just what a doc says)
  → **Yes, widen it to `--all-targets`.** Raised explicitly and approved. The
  deciding argument is a real hole rather than tidiness: the same rule permits
  committing a trivial change **directly to whichever branch is checked out**,
  which can be `develop`, and CI triggers on `push: branches: [main, develop]`.
  That makes this the only path in the workflow where the documented local gate
  can pass while CI fails, with no PR in between — so a narrower L47 doesn't
  keep the loop lighter, it relocates the failure to `develop`'s CI badge and
  costs a second commit to clear.
  **Measured cost** (isolated target dirs, 2026-08-05): cold run 4.74s → 5.30s
  user time; incremental runs indistinguishable. Sub-second.
  **Accepted downside:** a future clippy release could fire a new lint on
  existing test code and block an unrelated one-line fix. Weak in practice —
  after task 1 the tree is clean under `--all-targets`, and CI would reject the
  same push regardless, so the wider gate surfaces that failure rather than
  creating it.
  **Rejected alternative,** recorded so it is not re-proposed: keep L47 bare and
  add a clause requiring `--all-targets` only when committing directly to
  `develop`/`main`. It closes the same hole but adds a conditional rule to the
  workflow, which costs more attention than the sub-second it saves.

---

## Tasks

### Process notes

Carried forward from the plan template's HTML comments, which are easy to lose
once the template is filled in:

- **Plan status flips:** `DRAFT` → `APPROVED` *before* the task-0 commit;
  → `IN PROGRESS` on the task-1 commit, committed alongside that task's changes;
  → `READY` in the Finish checklist.
- **Draft PR:** task 0's plan-file commit is the first push. Open the draft PR
  against `develop` immediately after, with the AGENTS.md §2 header block
  (`Plan:` / `Source: #20` / `Status: WIP — do not review yet`) plus the
  `## What` / `## Why` / `## Notes for reviewer` sections from
  `docs/git-workflow.md`. Use `ref #20`, **not** `Closes #20`, while it is a
  draft — and see the Release section on why `Closes #20` never goes on this PR
  at all.
- **`_PR:_` header:** fill in `_PR:_ #<n>` once the draft PR exists and commit it.
- **One commit per task**, subject `<type>: <description> (task <N>, ref #20)`.
  Push after every 2 completed tasks, before any risky git operation, and before
  ending a session.
- **Issue labels — current state, so no one re-does completed steps.** As of
  2026-08-05 #20 carries exactly `enhancement` + `needs-review`; `needs-decision`
  was already removed when this plan was submitted, because every blocking
  question is resolved. The next transition is the **maintainer's**: on approval,
  remove `needs-review` and add `agent`. The agent then adds `in progress` at the
  start of task 1 (`gh issue edit 20 --add-label "in progress"`) and keeps
  `agent` alongside it.

### Implementation

- [x] 0. Create branch `chore/issue-20-public-release-readiness` from develop following docs/git-workflow.md
      **Run this from the primary worktree `/Users/marco/Build/mdv`**, which is
      where `develop` is checked out. Two other worktrees exist on unrelated
      branches (`chore/issue-24-…`, `fix/issue-18-…`); starting from one of those
      would branch off the wrong base. Confirm with `git branch --show-current`
      → `develop` before creating the branch. This plan file is untracked and
      survives the checkout.
- [x] 1. Fix the two `clippy::bool_assert_comparison` warnings in `src/layout.rs`
      — lines 1048 and 1066, both `assert_eq!(result.lines[0].spans[0].style.bold, true)`
      → `assert!(result.lines[0].spans[0].style.bold)`. These are the deferred LOW
      finding from `docs/plans/PLAN-phase3-structural-blocks.md`; that plan's
      Deferred findings entry should be left as-is (it is a historical record of
      that plan, not a live TODO) — this plan is where the fix is recorded.
      **Line numbers are from 2026-08-05 and may have moved** — locate them by
      running `cargo clippy --all-targets --locked` and following the diagnostic,
      not by trusting the numbers here.
      **Verify the fix changes nothing but style:** run `cargo test` before and
      after and confirm the pass count is identical. `assert_eq!(x, true)` and
      `assert!(x)` differ only in the failure message, so any change in count or
      outcome means something else was touched.
- [x] 2. Switch the CI clippy job to `--all-targets`:
      `.github/workflows/ci.yml`, clippy job, `cargo clippy --locked -- -D warnings`
      → `cargo clippy --all-targets --locked -- -D warnings`. Keep `--locked`
      (it is what makes CI reproduce the committed `Cargo.lock`) and keep the
      three-OS matrix.
      **The local run is not the verification for this task.** `src/main.rs`
      carries `#[cfg]`-gated Windows code that never compiles on macOS, so
      `--all-targets` could surface warnings on the Windows runner that no local
      run can predict. The draft PR's CI run across all three OSes is the real
      check. **If a platform-specific warning appears, fix it — do not defer it
      and do not relax the flag to make CI green**; it is the same class of gap
      this task exists to close.
      **Where the fix goes depends on when it surfaces.** CI results for this
      change arrive after task 2 is already committed, and possibly not until
      task 7. If it surfaces while task 2 is still open, fix it here. If it
      surfaces later, **add a new numbered task** to this Implementation section
      and complete it there — the same mechanism AGENTS.md §4 prescribes for
      adversarial-review findings. Do not silently amend a checked-off task, and
      do not tack the fix onto an unrelated task's commit.
      Leave the `fmt`, `test` and `audit` jobs untouched; `cargo fmt --check`
      already covers all targets.
- [x] 3. Bring the *documented* clippy command into line with what CI now runs.
      Otherwise a change passes locally and fails CI — precisely the trap
      `CONTRIBUTING.md` already warns about for `Cargo.lock`.
      **All 6 sites, enumerated** (verified 2026-08-05; line numbers may drift,
      so match on the command text, not the number). Every one becomes
      `cargo clippy --all-targets -- -D warnings`, preserving each site's
      existing comment and column alignment:

      | File | Line | Context |
      |---|---|---|
      | `AGENTS.md` | 26 | Commands fenced block, trailing `# must be clean after every phase` |
      | `AGENTS.md` | 47 | Trivial-change rule, prose, inline code — **see note below** |
      | `AGENTS.md` | 269 | Code style bullet, "Idiomatic Rust", inline code |
      | `CONTRIBUTING.md` | 19 | PR-bar fenced block |
      | `README.md` | 90 | `## Development` fenced block, trailing `# must be clean` |
      | `docs/mdv-build-plan.md` | 311 | Phase acceptance criteria, inline code |

      **`AGENTS.md:47` is the one site that changes what the maintainer actually
      runs**, not just what a doc says — it is the trivial-change gate for a
      one-line fix committed with no plan, branch or PR. Widening it was raised
      with the maintainer and **approved**; the reasoning, the measured cost, and
      the rejected alternative are in Open questions. Do not narrow it back on the
      grounds that it is "stricter than needed" — closing that hole is the point.
      **`docs/mdv-build-plan.md:311` is included deliberately** — it is live
      specification stating the gate every build phase must pass, not a record of
      a past run, so leaving it stale would contradict both CI and `AGENTS.md`.
      It is the one site where the call could reasonably go either way; it is
      decided here so the agent does not have to guess.
      **`docs/plans/*.md` are deliberately excluded** — five plan files mention
      the command, but each records what was actually run at the time, and
      editing them would falsify the implementation record. That includes
      `PLAN-phase3-structural-blocks.md`, whose Deferred findings entry names
      this exact gap and says it is "left for a future" fix; that entry stays
      untouched as the historical note it is. **This plan is where the fix is
      recorded.**
      **Then confirm nothing was missed:**
      ```bash
      grep -rn "cargo clippy" --include="*.md" --include="*.yml" . \
        | grep -v "^./target" | grep -v "docs/plans/"
      ```
      Every surviving hit must read `--all-targets`, and only
      `.github/workflows/ci.yml` additionally carries `--locked` (task 2).
- [x] 4. Mark `AGENTS.md`'s two audiences **in place**. Two insertions, no moves,
      no renumbering, no rule text touched. **Exact wording is given below** so
      the maintainer approves the actual words at plan-review time rather than
      the agent inventing them; adjust only for typos or house style.
      - **Audience banner** — insert as a third blockquote after the existing
        `> **PROJECT:** … > **STACK:** …` block (i.e. between current L6 and the
        `---` on L8). Keep both existing blockquotes exactly as they are:
        ```markdown
        > **Audience:** Everything in this file applies to anyone working on
        > `mdv` — **except `## Workflow (required)` (§1–§5)**, which is the
        > maintainer's own process (plan files, GitHub label state machine,
        > review ritual) and is not expected of outside contributions.
        > Contributing from outside? Start with `CONTRIBUTING.md`.
        ```
      - **Divider** — insert immediately after the `## Workflow (required)`
        heading, before `### 1. Development phases`:
        ```markdown
        > **This section is the maintainer's process.** It is how the maintainer
        > — and any agent working on their behalf — runs changes in this repo,
        > and it is required in that context. An external contributor does not
        > need to write a plan file, follow the branch naming, or touch issue
        > labels; see `CONTRIBUTING.md` for what an outside PR actually needs.
        ```
      **Scope of the marking is `## Workflow (required)` and nothing else.**
      The banner deliberately does **not** enumerate the universal sections. An
      earlier draft listed them and silently omitted `## Agent behavior` and
      `## Evolving this file`, which would have forced the agent to guess which
      side those fall on. The rule is simply: `## Workflow (required)` (§1–§5) is
      marked; every other section is unmarked and therefore universal. Do not add
      markers, headings, or "maintainer-only" notes to any other section.
      **Hard constraints — the whole point of this task is that it changes
      nothing operative:**
      - Do **not** move any section, do **not** renumber §1–§5, do **not** reword
        any rule inside them. `docs/git-workflow.md:19,226,239` cite "AGENTS.md
        §1 step 4" and "§2", and `docs/plans/PLAN-template.md:6` cites "§5" —
        all four must still resolve, unchanged, after this task.
      - The word "required" stays in the `## Workflow (required)` heading. It is
        required *of the maintainer*, and the divider says so; softening the
        heading would weaken the rule in the maintainer's own sessions, which is
        the opposite of this task's intent.
      - Do **not** touch `CLAUDE.md`. Its `@AGENTS.md` import is what loads these
        rules into a session; leaving it alone is what guarantees zero workflow
        impact.
      - Do **not** edit `CONTRIBUTING.md:28`. It refers to `AGENTS.md`
        generically with no section anchor — deliberately, per #24 — and stays
        accurate under this decision. (Task 3 edits `CONTRIBUTING.md`'s fenced
        command block only; this sentence is untouched.)
      **Verify by diff:** `git diff AGENTS.md` for this task must show additions
      only — no deletions inside the `## Workflow` block. Any deletion there
      means the task overreached.
- [x] 5. Delete `docs/api-error-handling.md` and remove its one inbound
      reference in the **same commit**, so no state exists where `AGENTS.md`
      points at a file that isn't there. The reference is the **first bullet of
      `## Domain Specific Rules`**, `AGENTS.md:284–285`. **Exact edit:**
      ```markdown
      - This is a single Rust binary crate — there are no frontend/backend layers,
        no API contracts, and no `docs/api-error-handling.md` applicability.
      ```
      becomes
      ```markdown
      - This is a single Rust binary crate — there are no frontend/backend layers
        and no API contracts.
      ```
      The second bullet of that section (`docs/mdv-build-plan.md` is the complete
      specification…) is untouched.
      **Before deleting, re-run the reference check** rather than trusting this
      plan's grep from 2026-08-05:
      `grep -rn "api-error-handling" --include="*.md" --include="*.yml" --include="*.rs" . | grep -v "^./target"`
      If anything outside `AGENTS.md` and this plan turns up, handle it in this
      task.
      **Also confirm `git status` shows `docs/session-retrospective-phase3-phase4.html`
      still untracked** — it stays that way by deliberate choice per #20, and a
      `git add docs/` in this task is the one plausible way it gets swept in.

### Verification

- [x] 6. Run the full local gate exactly as CI now defines it:
      `cargo fmt --check`, `cargo clippy --all-targets --locked -- -D warnings`,
      `cargo test --locked`. All three clean, test count unchanged from the
      task-1 baseline.
- [x] 7. Confirm the draft PR's CI is green **on all three OSes** — this is the
      only place the widened clippy scope is actually proven on Windows and
      Linux. `gh pr checks <pr>`. A local-only pass does not close task 2.
- [x] 8. Re-read the changed docs end to end for accuracy: `AGENTS.md`'s banner
      and divider read as *describing* the two audiences rather than changing any
      rule; no doc still names `docs/api-error-handling.md`; and the clippy
      command is identical across **all 6 sites from task 3** plus
      `.github/workflows/ci.yml` (which additionally carries `--locked`) — i.e.
      `AGENTS.md` ×3, `CONTRIBUTING.md`, `README.md`, `docs/mdv-build-plan.md`.

### Finish

- [ ] Write / update tests for all implementation tasks above
      — expected **N/A**: task 1 rewrites two existing assertions without
      changing what they assert, and tasks 2–5 touch CI config and docs only.
      Record it here explicitly rather than skipping it silently.
- [ ] Run full test suite — all tests pass
- [ ] `cargo audit` — expected **N/A**: `AGENTS.md` requires it after the first
      build and after any `cargo update`, and this plan changes no dependency and
      does not touch `Cargo.toml` or `Cargo.lock`. Stated rather than skipped
      silently. If `Cargo.lock` did change for any reason, run it and treat any
      advisory as a failure.
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected — task 3 already does; confirm nothing else
      in it went stale.
- [ ] Convert draft PR to ready-for-review; set this plan's `_Status:_` to `READY`.
      **Do not add `Closes #20` to this PR** — see the Release section. #20 is
      not finished when this PR merges; the promotion and the visibility flip
      are still outstanding, and auto-closing it here would lose them.
      **`READY` describes this PR, not #20.** It is the terminal status the plan
      file records (per the template), and it is correct to set it with R1–R4
      still unchecked — those are tracked on #20, which stays open until R4.
      Do not withhold `READY` waiting for the release steps, and do not invent a
      status beyond it.
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 20 --remove-label agent --remove-label "in progress" --add-label needs-review`

---

## Release — after the develop PR merges

These steps are **not** part of the feature branch and must not be attempted
before the PR above is merged into `develop`. Ordering matters: `main` should be
promoted only once `develop` contains the finished work, and the repo should go
public only once `main` is the project rather than the `init` commit.

- [ ] R1. **Promote `develop` → `main` via a release PR.** Not a direct push —
      `docs/git-workflow.md:5` forbids committing directly to `main` or
      `develop`, and CI is configured to run on PRs targeting `main`, so the PR
      is also what gets the promotion tested.
      ```bash
      git fetch origin
      git log --oneline origin/main..origin/develop   # expect the full project history
      # write the body below to a scratch file first, then:
      gh pr create --base main --head develop \
        --title "Release: promote develop to main" \
        --body-file <scratch-file>
      ```
      **PR body** — write this to a scratch file rather than inlining it, so the
      plan file's indentation never leaks into the rendered PR:
      > Brings `main` up to date with `develop` ahead of making the repo public.
      > Ref #20.
      >
      > **Merge with "Create a merge commit" — not squash, not rebase.** Squashing
      > collapses the whole project history into one commit on `main`, which is
      > the history curation #20 explicitly rejected. Rebasing rewrites every
      > commit under new SHAs. Both permanently diverge `main` from `develop`.
      `main` is a strict ancestor of `develop`, so there is nothing to resolve.
      **Confirm that is still true before opening the PR** — if `git log
      origin/develop..origin/main` returns anything, `main` has diverged since
      2026-08-05 and this step needs re-planning, not force-merging.
      **Merge is the maintainer's action**, as with any PR in this repo.
      **Merge method: "Create a merge commit". Not squash, not rebase.**
      All three buttons are enabled on this repo, and the choice is not cosmetic:
      - **Squash** would collapse all 13 commits into a single commit on `main`,
        which is *literally* the history curation #20 considered and rejected.
        It would also permanently diverge `main` from `develop`, breaking every
        future promotion.
      - **Rebase** rewrites the 13 commits with new SHAs on `main`, duplicating
        the entire history under different identities — same divergence problem.
      - **Merge commit** preserves every commit as-is and is the only option
        consistent with "promote as-is, no curation".
      The PR body above states this deliberately, so whoever clicks the button —
      the maintainer now, or a future session — does not pick from the dropdown
      by habit. Keep that paragraph in the body; it is the safeguard, not filler.
      **Expected and fine:** after the merge, `main` carries one merge commit
      that `develop` does not. That is normal for this two-branch model and does
      not constitute divergence — `git diff` between the branches stays empty
      (R2 checks the diff, deliberately, not the log). Merging `main` back into
      `develop` is **not** required.
- [ ] R2. **Verify `main` after the merge:** `git diff origin/main origin/develop`
      is empty, `README.md` and `src/` are present on `main`, and CI on `main` is
      green.
- [ ] R3. **Flip visibility Private → Public.** *Maintainer action — the agent
      does not run this.* It exposes the full repo and history to the internet
      and is not meaningfully reversible (clones, forks and caches persist).
      ```bash
      gh repo edit mrcmilano/mdv --visibility public --accept-visibility-change-consequences
      ```
      Before running it, re-confirm what goes public with it — every branch, the
      full commit history including the personal email in early commits, all
      `docs/plans/` files, `AGENTS.md` and `CLAUDE.md`. All of that was reviewed
      and accepted in #20; this is the last checkpoint, not a new decision.
- [ ] R4. **Close #20** once R3 is done — *maintainer action*, for the same
      reason R3 is: the agent never performs the flip, so it is never in a
      position to confirm the issue is actually complete. Then note that **#30**
      (post-launch hygiene — branch protection, topics, Discussions) is
      unblocked, since branch protection only becomes configurable after the
      flip.
