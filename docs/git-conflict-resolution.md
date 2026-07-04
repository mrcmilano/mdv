# Conflict Resolution Prompt — Feature Branch → Develop

Use this prompt when a PR has conflicts that must be resolved before merging.

---

## Context to establish before starting

- **Branch:** the feature branch with open PR
- **Target:** `develop`
- **Goal:** resolve all conflicts, keep both sides' intent intact, push a clean branch ready to merge

---

## Step 1 — Understand before touching anything

```bash
git fetch origin
git checkout feature/your-branch
git log --oneline origin/develop..HEAD     # commits unique to your branch
git diff origin/develop...HEAD             # what your branch changed vs develop
```

Before proceeding, explicitly answer:
1. What is this branch trying to achieve?
2. What has `develop` received since this branch was created?
3. Are the conflicting changes touching the same concern, or just the same file incidentally?

Do not proceed to resolution until you can answer all three.

---

## Step 2 — Rebase onto develop

```bash
git rebase origin/develop
```

List every conflicting file:

```bash
git diff --name-only --diff-filter=U
```

---

## Step 3 — Resolve each conflict explicitly

Work through files one at a time. For each conflict hunk:

1. **Read the incoming side** (`<<<<<<< HEAD` / `develop`) — what changed and why?
2. **Read your side** (`=======` / `>>>>>>>`) — what does the branch change and why?
3. **Determine resolution:**
   - Changes are independent → keep both
   - Changes overlap → produce a result that satisfies both intents
   - Intent is ambiguous → **stop, do not guess** (see Stop Conditions below)

```bash
git add <resolved-file>
git rebase --continue
```

**Never:**
- Accept one side wholesale without reading the other
- Produce a resolution that compiles or runs but hasn't been reasoned through
- Leave any conflict marker in the file (`<<<<<<<`, `=======`, `>>>>>>>`)

---

## Step 4 — Verify

```bash
git diff origin/develop..HEAD              # full diff of what the PR will introduce
git log --oneline origin/develop..HEAD     # confirm your commits are all present
```

Run the test suite if available. A resolution that breaks tests must not be pushed — return to Step 3.

---

## Step 5 — Push and update the PR

```bash
git push --force-with-lease origin feature/your-branch
```

Then add a comment to the PR describing:
- Which files had conflicts
- What the conflict was in plain terms
- How it was resolved and why

This is not optional — reviewers need to verify your resolution decisions, not discover them silently in the diff.

---

## Stop Conditions — escalate to a human if any of these are true

- The two sides address the same logic from different directions and the correct merge is genuinely ambiguous
- Resolving correctly requires understanding business logic or product decisions you don't have full context for
- Tests fail after resolution and the cause is not immediately clear
- More than 3–4 files are in conflict (compounding error risk is high)
- You have aborted and retried the rebase more than once

When in doubt: `git rebase --abort` to return to the pre-rebase state, then flag for human review. Do not push a resolution you are not confident in.

---

## A note on rebase vs merge

This prompt uses rebase deliberately. Merging `develop` into the feature branch would avoid the force-push but adds a merge commit that pollutes the PR diff and makes review harder. Rebase keeps the history linear and the diff clean.

Exception: if reviewers have left detailed inline comments that a force-push would displace, flag this before pushing and confirm with the reviewer first.