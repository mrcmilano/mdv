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
cargo build                           # debug build
cargo build --release                 # release build (lto + strip, see Cargo.toml)
cargo test                            # all tests
cargo test [test_name]                # single test
cargo clippy -- -D warnings           # must be clean after every phase
cargo fmt                             # format
cargo audit                           # run after first build and after any cargo update
```

---

## Workflow (required)

### 1. Development phases

The required sequence is: **Assess → Plan → Branch → Implement → Finish**. Do not skip or reorder steps.

A change is **trivial** only if it has zero logic impact AND is limited in scope
(one-liner, typo, `.md` wording). When in doubt, treat as non-trivial.

For non-trivial work:

1. **Assess** — evaluate change request impact. If anything is unclear, ask first. Do not create a branch or write any code yet.

2. **Plan** — create `docs/plans/PLAN-<feature-name>.md` using the template at
   `docs/PLAN-template.md`. Stop once the plan is written.
   **Do not create a branch or implement until the user confirms approval.**
   Every plan must include as its first task:
   `` - [ ] 0. Create branch `<branch-name>` from develop following docs/git-workflow.md ``
   The branch name goes in this task, not only in the plan header.

3. **Branch** — once the plan is approved, read `docs/git-workflow.md` in full,
   then create a new branch from **develop**. This is mandatory — there are no exceptions
   based on scope, triviality, or whether a commit is imminent. Do not write any
   implementation code before the branch exists.

4. **Implement** — confirm you are on the correct feature branch before writing a single line.
   If not, stop and resolve this before continuing. Work one task at a time per the approved
   plan, running the test loop after each task. Mark each plan task `[x]` when complete and
   tests pass. The plan file is the source of truth for remaining work — if context is lost,
   re-read it before continuing.

5. **Finish** — invoke the `adversarial-review` skill. Fix all findings before continuing. 
   If approved, update `README.md` if anything is now outdated.

### 2. Git workflow

**After the first push on a new branch**, open a draft PR targeting `develop` immediately — do not wait until the work is complete.

**Before any subsequent git operation** — committing, pushing, opening a PR, updating a branch,
or handling a merge — consult the relevant section of `docs/git-workflow.md` before proceeding.

**If a PR has conflicts** that must be resolved before merging, additionally load:

→ `docs/git-conflict-resolution.md`

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

## Code style & conventions

- Match the conventions already in a file over your own defaults.
- Prefer small, focused modules over large multi-purpose ones — the module
  layout is fixed by `docs/mdv-build-plan.md` Section 4 (`main.rs`, `render.rs`,
  `layout.rs`, `view.rs`, `input.rs`, `style.rs`); do not add new top-level
  modules without updating that section first.
- Idiomatic Rust: run `cargo fmt`; `cargo clippy -- -D warnings` must be clean.
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