# AGENTS.md — Project Blueprint

> **Note:** This is a **minimum** set of rules. It's a living document that needs to be updated as the project evolves. Always ask if unsure.

> **PROJECT:** `[one-line description — replace me]`
> **STACK:** `[fill in as decisions land — e.g. Python/FastAPI + React/TypeScript]`

---

## Setup

### Package management

```bash
# Python (uv)
test -d .venv && source .venv/bin/activate
uv sync
```

```bash
# JS/TS
npm ci
```

## Commands

```bash
# Python
uv run pytest                         # all tests
uv run pytest [path::test]            # single test
uv run ruff check .
uv run mypy [src/]
```

```bash
# JS/TS
npm test                              # all tests
npm run lint
npm run typecheck
npm run build
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
   `docs/plans/PLAN-template.md`. Stop once the plan is written.
   **Do not create a branch or implement until the user confirms approval.**
   Every plan must include as its first task:
   `- [ ] 0. Create branch \`<branch-name>\` from develop following docs/git-workflow.md`
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

5. **Finish** — run `/skill:adversarial-review`. Fix all findings before continuing. 
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

- Mock all I/O, network, and database calls in unit tests.
- Tests must be deterministic and isolated — no shared state between tests.
- Test coverage should make the behaviour of the code readable without
  opening the implementation.
  
---

## Code style & conventions

- Match the conventions already in a file over your own defaults.
- Prefer small, focused modules over large multi-purpose ones.
- **Python:** PEP 8, type hints on all public functions, Ruff for lint + format.
- **TS/JS:** strict types, no `any` without reason, follow existing lint/tsconfig.
- **Before adding any new dependency, stop and ask.**

## Architecture & boundaries

Records project structure decisions as they are made in the file `docs/architecture.md` . Capture each layer once established.
If it applies to the project, use this structure:

```
[backend/]  — Python API / services (e.g. app/routes/, app/services/)
[frontend/] — TS/JS client          (e.g. src/components/, src/pages/)
[shared/]   — shared schemas, types (e.g. shared/types.ts, shared/schemas/)
```

- Respect layer boundaries (e.g. routes → services → data).
- API contracts (schemas/types) are the source of truth across language boundaries — update both sides together.
- Replace the examples above with actual paths as the project grows.

## Domain Specific Rules

- If you are working on a project that uses a specific framework or library, follow the conventions and best practices of that framework or library.

## API error handling

If the task involves **writing or modifying frontend code that calls an API** — fetch calls, HTTP client usage, data-fetching hooks, form submissions, or any component that reads from or writes to an endpoint — read `docs/api-error-handling.md` before writing any code.
This includes: adding new API calls, handling error or loading states, retry logic, form error display, offline behaviour, or request cancellation.

## Security

- Never commit secrets or `.env` files. Read secrets from environment variables.
- Create `.env.example` with dummy values during initial setup; keep it in sync with `.env`.

## Agent behavior

- This file takes precedence over tool defaults when they conflict.
- Report progress concisely after each implementation step; do not narrate every keystroke.

## Evolving this file

Add a rule whenever the agent repeatedly gets something wrong or when a new pattern/structure is established. Keep it concise — remove sections that no longer apply.