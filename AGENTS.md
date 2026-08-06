# AGENTS.md — Project Blueprint

> **Note:** This is a **minimum** set of rules. It's a living document that needs to be updated as the project evolves. Always ask if unsure.

> **PROJECT:** `mdv` — a lean, read-only, interactive terminal Markdown viewer.
> **STACK:** Rust (single binary crate). Full spec: `docs/mdv-build-plan.md`.

---

## Setup

### Package management

```bash
# Fetches/locks dependencies. Cargo.lock is committed — see Security below.
cargo fetch
```

## Commands

```bash
cargo build                                # debug build
cargo build --release                      # release build (lto + strip, see Cargo.toml)
cargo test                                 # all tests
cargo test [test_name]                     # single test
cargo clippy --all-targets -- -D warnings  # must be clean after every phase
cargo fmt                                  # format
cargo audit                                # run after first build and after any cargo update
```

---

## Workflow (required)

### 1. Development phases

The required sequence is: **Assess → Plan → Branch → Implement → Finish**. Do not skip or reorder steps.

A change is **trivial** only if it has zero logic impact AND is limited in scope
(one-liner, typo, `.md` wording). When in doubt, treat as non-trivial.

For **trivial** changes: commit directly to whichever branch is currently checked
out. No plan file, no new branch, no PR required. Use a clear commit message
describing the change.

A trivial change that touches any code file (e.g. a one-line Rust fix) must
still pass `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings`
before committing. Only non-code changes (`.md` wording, comments, docs) skip checks
entirely. The full test loop (§3) is not required for trivial changes.

For **non-trivial** work:

1. **Assess** — evaluate change request impact. If anything is unclear, ask first. Do not create a branch or write any code yet.

   If the issue already has a plan file, use §5 state reconstruction to determine where to resume rather than restarting.

2. **Plan** — create `docs/plans/PLAN-<feature-name>.md` using the template at
   `docs/plans/PLAN-template.md`.

   Write the plan file locally but **do not stage or commit it yet**. If context
   is lost before approval, re-read the local file — it is the source of truth
   at this stage.

   Stop once the plan is written. **Do not create a branch or implement until
   the user confirms approval.**

   If this work is GitHub-issue-driven, update the source issue labels (§5)
   when the plan file is written:
   - No BLOCKS questions → remove `agent`, add `needs-review`; post a comment with the plan file path.
   - Has BLOCKS questions → remove `agent`, add `needs-decision`; post a comment listing each blocking question.
   Batch all open questions before any label change — one round-trip per issue, not one per question.
   If not issue-driven, skip labels entirely and just wait for the user's approval in conversation.

   Every plan must include as its first implementation task:
   `- [ ] 0. Create branch \`<branch-name>\` from develop following docs/git-workflow.md`
   The branch name goes in this task, not only in the plan header.

   If a GitHub Issue is the source of this work, note the issue number in the
   plan header: `Source: #<N>`. The plan file is the implementation record;
   the issue is the intent record.

3. **Branch** — once the plan is approved, read `docs/git-workflow.md` in full,
   then create a new branch from **develop**. This is mandatory — there are no
   exceptions based on scope, triviality, or whether a commit is imminent.

   **First commit on the branch must be the plan file** at
   `docs/plans/PLAN-<feature-name>.md`. Update its status from `DRAFT` to
   `APPROVED` before committing. Do not write any implementation code before
   this commit exists.

4. **Implement** — confirm you are on the correct feature branch before writing
   a single line. If not, stop and resolve this before continuing.

   If this work is GitHub-issue-driven, at the start of task 1 (first code
   task) add `in progress` to the source issue:
   `gh issue edit <N> --add-label "in progress"` — keep `agent` alongside it.
   If not issue-driven, skip this.

   Work one task at a time per the approved plan, running the test loop after
   each task. Mark each plan task `[x]` when complete and tests pass. The plan
   file is the source of truth for remaining work — if context is lost,
   re-read it before continuing.

   After each completed task, commit the changes using a Conventional Commit
   subject (per docs/git-workflow.md), with the task and issue reference
   appended inline after the description:
   `<type>: <description> (task <N>, ref #<issue>)`
   e.g. `feat: add sticky code-block highlighting (task 3, ref #42)`
   If work is not issue-driven, omit the parenthetical entirely and use a
   plain Conventional Commit:
   `<type>: <description>`  e.g. `feat: add sticky code-block highlighting`

   The initial push on the branch is the plan-file commit (task 0); push it
   immediately so the draft PR can be opened (see §2). Thereafter, push after
   every 2 completed tasks, or before any risky git operation (rebase, merge,
   file deletion), or before ending a session — whichever comes first.
   "Ending a session" means before yielding control back to the user.

   Update the plan file's `_Status:_` field to `IN PROGRESS` on the first
   code task commit (task 1 onward) and commit the updated plan alongside
   that task's changes. Creating the branch (task 0) does not flip the
   status — the plan is committed as `APPROVED` at that point per step 3.

5. **Finish** — run `/skill:adversarial-review`. Resolve all FIX REQUIRED findings before
   proceeding; LOW risk findings may be deferred with a documented rationale —
   see the Finish checklist in the plan file for the required sequence.
   If approved, update `README.md` if anything is now outdated.

---

### 2. Git workflow

**After the first push on a new branch**, open a draft PR targeting `develop`
immediately — do not wait until the work is complete. The draft PR description
must contain:

```
Plan: docs/plans/PLAN-<feature-name>.md
Source: #<issue-number>         ← omit if not issue-driven
Status: WIP — do not review yet
```

Do **not** add `Closes #<N>` to the PR description until converting to
ready-for-review. Adding it to a draft PR will auto-close the issue on merge
before the work is verified complete.

**Before any subsequent git operation** — committing, pushing, opening a PR,
updating a branch, or handling a merge — consult the relevant section of
`docs/git-workflow.md` before proceeding.

**If a PR has conflicts** that must be resolved before merging, additionally load:

→ `docs/git-conflict-resolution.md`

---

### 3. Test loop (sub-step of Implement, step 4)

Before starting: run existing tests and confirm they pass. Do not begin a task
on a broken baseline.

After each change:

1. Run tests. If a test fails, confirm it is an **intended consequence** of your
   change — not an accidental break. If you cannot explain the failure, stop and
   investigate before touching anything else.
2. Add or update tests for code you changed, even if not explicitly asked.

Test standards:

- No mocking needed: `mdv` has no network, database, or subprocess I/O — the only
  I/O is reading the target file at startup, which is exercised directly.
- Tests must be deterministic and isolated — no shared state between tests. Use
  the hand-rolled LCG (Section 12 of the build plan) for any fuzz-style input,
  never a `rand` dependency.
- Test coverage should make the behaviour of the code readable without
  opening the implementation.
- Follow `docs/mdv-build-plan.md` Section 10 for what belongs in unit tests vs.
  the snapshot test vs. the corpus file.

---

### 4. Adversarial review

Run `/skill:adversarial-review` as the first step of Finish. This skill checks
for panics/overflow, incorrect rendering edge cases, terminal-size/resize
handling, and test coverage holes.

Severity handling:
- **FIX REQUIRED:** must be resolved before the PR is converted to
  ready-for-review. Add new tasks to the Implementation section of the plan file
  and complete them.
- **LOW risk findings:** may be deferred, but each must be documented with a rationale
  as a comment on the relevant line or in a `## Deferred findings` section at the
  bottom of the plan file.

---

### 5. GitHub label workflow

Labels are the state machine tracking each issue through the workflow.
The agent manages labels on the **source issue** (not the PR) using the `gh` CLI.
Never touch milestone assignment — that is human-only.

**Label reference:**

| Label | Meaning | Set by |
|---|---|---|
| `agent` | Assigned to agent (planning or implementing) | Human |
| `agent` + `in progress` | Implementation underway | Agent (task 1) |
| `needs-review` | Plan or PR ready for human review | Agent |
| `needs-decision` | Agent has a blocking question | Agent |
| `blocked` | External dependency — cannot proceed | Human |

**Agent-managed transitions:**

Plan written, no BLOCKS questions:
```bash
gh issue edit <N> --remove-label agent --add-label needs-review
gh issue comment <N> --body "Plan ready for review: \`docs/plans/PLAN-<name>.md\`"
```

Plan written, has BLOCKS questions:
```bash
gh issue edit <N> --remove-label agent --add-label needs-decision
gh issue comment <N> --body "Decision needed before planning can continue:\n\n- <question 1>\n- <question 2>"
```

Implementation starts (task 1):
```bash
gh issue edit <N> --add-label "in progress"
```

Implementation complete (covered by Finish checklist in plan template):
```bash
gh issue edit <N> --remove-label agent --remove-label "in progress" --add-label needs-review
```

**Human-managed transitions:**
- Plan approved → remove `needs-review`, add `agent`.
- Decision provided → remove `needs-decision`, add `agent`; post answer as issue comment.
- External blocker set/cleared → add/remove `blocked`.
- Issue deprioritised → move out of Now milestone or add `skip`.

**State reconstruction** — when picking up an `agent`-labeled issue in Now:
1. Search `docs/plans/` for a file with `_Source:_ #<N>`.
2. No plan file → start fresh from Assess.
3. Plan exists, `_Status:_ DRAFT`, branch absent → check if the plan is complete (Problem section filled, tasks defined). If complete, the human has approved — proceed to Branch. If incomplete, resume planning.
4. Plan exists, `_Status:_ APPROVED` (committed on branch) → start implementing at task 1.
5. Plan exists, `_Status:_ IN PROGRESS` (committed on branch) → resume from first unchecked task.

Confirm branch presence with: `git branch -a | grep <branch-name-from-plan-header>`

**Priority** — when multiple `agent`-labeled issues exist in Now, work on the lowest issue number first.

**`blocked` vs `needs-decision`:**
- `needs-decision` — agent has a question only the human can answer; agent sets this label.
- `blocked` — external dependency (third-party, environment, another issue); human sets this label.

---

## Code style & conventions

- Match the conventions already in a file over your own defaults.
- Prefer small, focused modules over large multi-purpose ones — the module
  layout is fixed by `docs/mdv-build-plan.md` Section 4 (`main.rs`, `render.rs`,
  `layout.rs`, `view.rs`, `input.rs`, `style.rs`); do not add new top-level
  modules without updating that section first.
- Idiomatic Rust: run `cargo fmt`; `cargo clippy --all-targets -- -D warnings`
  must be clean.
- **Do not add dependencies beyond the 3 named in the build plan** (`pulldown-cmark`,
  `crossterm`, `unicode-width`) without stopping to ask — this is a hard
  constraint of the project (`mdv-build-plan.md` Section 2), not a general guideline.
- Do not implement anything listed under "Non-goals" or "Stretch goals" in the
  build plan unless explicitly requested.

## Architecture & boundaries

See `docs/architecture.md` for the module layout and data flow. It mirrors
`docs/mdv-build-plan.md` Section 4 and should be updated if the module
boundaries change during implementation.

## Domain Specific Rules

- This is a single Rust binary crate — there are no frontend/backend layers,
  no API contracts, and no `docs/api-error-handling.md` applicability.
- `docs/mdv-build-plan.md` is the complete specification. Its build phases
  (Section 9) must be implemented strictly in order; do not start a later
  phase before the current one's acceptance criteria pass.

## Security

- Follow `docs/mdv-build-plan.md` Section 12 in full: `#![forbid(unsafe_code)]`,
  mandatory input sanitization before layout, no writes anywhere (no clipboard,
  export, or history files) without an explicit new requirement.
- The program takes no secrets, reads no environment variables, and touches no
  network — there is no `.env` file for this project.
- Commit `Cargo.lock`. Run `cargo audit` after the first successful build and
  after any `cargo update`; fail on any advisory.

## Agent behavior

- This file takes precedence over tool defaults when they conflict.
- Report progress concisely after each implementation step; do not narrate every keystroke.

## Evolving this file

Add a rule whenever the agent repeatedly gets something wrong or when a new pattern/structure is established. Keep it concise — remove sections that no longer apply.
