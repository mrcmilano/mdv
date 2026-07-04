# Git Workflow Instructions

## Core Principles

- **Never commit directly to `main` or `develop`.** All implementation work happens on a dedicated branch.
- Keep commits small, atomic, and focused on a single concern.
- Follow the Conventional Commits specification: `<type>[optional scope]: <description>`. Types: `feat:` (MINOR), `fix:` (PATCH), `BREAKING CHANGE` or `!` after type/scope (MAJOR), plus `build:`, `chore:`, `ci:`, `docs:`, `style:`, `refactor:`, `perf:`, `test:`, `hotfix:`. Use imperative mood in descriptions.

### Example Commit Messages

```feat: add user authentication```
```fix: resolve login redirect issue```
```docs: update README with new setup instructions```
```chore: update dependencies```

---

## Branch Strategy

| Branch | Purpose |
|---|---|
| `main` | Production-ready code only |
| `develop` | Integration branch; base for all new work |
| `feature/*` | New features |
| `fix/*` | Bug fixes |
| `chore/*` | Tooling, deps, refactoring, non-functional changes |

**Always branch from `develop` unless explicitly told otherwise.**

```bash
git checkout develop
git pull origin develop
git checkout -b feature/your-feature-name
```

Use short, lowercase, hyphenated names: `feature/user-auth`, `fix/login-redirect`.
Append ticket number when one exists (`feature/issue-42-user-auth`).

---

## Daily Workflow

```bash
# 1. Start from an up-to-date develop
git checkout develop && git pull origin develop

# 2. Create your branch
git checkout -b feature/my-change

# 3. Work, then stage and commit
git add -p                               # stage hunks interactively, not blindly
git commit -m "Add concise description of change"

# 4. Push and open a draft PR targeting develop
git push -u origin feature/my-change
```

---

## Keeping Your Branch Up to Date

Rebase onto `develop` before the first commit of any working period, and always before marking a PR ready. Do not let divergence accumulate.

```bash
git fetch origin
git rebase origin/develop
```

Prefer rebase over merge to keep history linear.

---

## Handling Problems

### Merge / Rebase Conflicts

**Before touching any conflicting file: understand what both sides intended. If the intent of either side is ambiguous, stop and escalate — do not guess.**

```bash
# 1. Inspect what is conflicting
git diff --name-only --diff-filter=U

# 2. Resolve each file, then mark it resolved
git add <resolved-file>

# 3. Continue
git rebase --continue      # or git merge --continue
```

- Resolve conflicts in the **smallest possible scope** — change only what is necessary to reconcile the two sides. Do not silently accept either side wholesale or make unrelated edits while resolving.
- If a rebase becomes too tangled: `git rebase --abort` to return to the pre-rebase state, then reassess before retrying.

### Broken Rebase State

If a rebase was interrupted (crash, timeout, lost context), the repo may be left mid-rebase with a `REBASE_HEAD` ref. New git commands will fail with confusing errors. Always check state before acting:

```bash
git status                 # will explicitly say "rebase in progress" if so
git rebase --abort         # safest recovery — returns to pre-rebase state cleanly
```

Only retry the rebase once you understand what caused the interruption.

### Accidental Commits to the Wrong Branch

```bash
# Move the last commit to a new branch
git branch feature/rescue-branch
git reset --hard HEAD~1
git checkout feature/rescue-branch
```

For multiple commits, use `git log --oneline` to identify the target SHA, then `git reset --hard <sha>` before creating the rescue branch.

### Undoing a Commit (not yet pushed)

```bash
git reset --soft HEAD~1   # keeps changes staged
```

### Reverting a Commit (already pushed / shared)

```bash
git revert <commit-sha>   # creates a new undo commit — never force-push shared history
```

### PR Merged onto the Wrong Branch (e.g. `main` instead of `develop`)

Always use `git revert -m 1` — never `reset --hard`. By the time a merge lands on `main`, CI/CD pipelines and other collaborators have likely already seen it. Rewriting history is unsafe; a revert commit is the correct response.

```bash
# 1. Identify the bad merge commit SHA
git log --oneline -5 origin/main

# 2. Revert it — -m 1 restores main's side (parent 1), discarding the feature branch changes
git checkout main
git pull origin main
git revert -m 1 <merge-commit-sha>
git push origin main

# 3. Verify main is clean
git log --oneline -5 origin/main
git diff origin/develop origin/main

# 4. The feature branch is untouched — open a new PR targeting develop
git checkout feature/your-branch
```

> `-m 1` is required for merge commits. It tells git which parent to restore: `1` is the branch merged *into* (`main`), `2` is the branch merged *from*. Without it, git cannot determine which side to revert to.

### Branched from Wrong Base (e.g. from `main` instead of `develop`)

Do not recreate the branch manually. Use `git rebase --onto` to replay your commits onto the correct base:

```bash
# Replay commits that are on feature/my-branch but not on main, onto develop
git rebase --onto develop main feature/my-branch

# Verify only your commits are present before pushing
git log --oneline develop..feature/my-branch

git push --force-with-lease origin feature/my-branch
```

### Detached HEAD and Lost Commits

A detached HEAD occurs when you check out a commit SHA directly instead of a branch name. Any commits made in this state are not on any branch and will appear lost after a checkout — but they are recoverable via `git reflog` for approximately 30 days.

**If you realise you are in detached HEAD, save your position immediately:**

```bash
git checkout -b feature/rescue-branch
```

**Recovering orphaned commits or a deleted branch:**

```bash
git reflog                               # find the SHA of the last commit on the lost work
git checkout -b feature/recovered <sha>
```

`git reflog` records every position HEAD has been at. Check it before concluding any commits are truly lost.

### Secret or Credential Accidentally Committed

This is the highest-consequence mistake in this document. **`git revert` is not sufficient** — the secret still exists in history and must be treated as compromised immediately, regardless of whether the branch has been pushed.

**Stop. Do the following in order:**

1. **Rotate the exposed credential immediately** — assume it has been seen.
2. Do not push the branch if it has not been pushed yet.
3. If already pushed, do not attempt self-recovery — stop and flag for human review.
4. History rewriting (`git filter-repo` or equivalent) is required to fully purge the secret, and it affects every collaborator. This must be a coordinated human decision, not an autonomous agent action.

> Never commit `.env` files, credentials, tokens, private keys, or any value that grants access. If uncertain about a file, check `.gitignore` before staging.

---

## Pull Requests

### Opening

- **Open a draft PR immediately after the first push** — do not wait until the work is done. This makes the branch visible, triggers CI early, and signals work-in-progress.
- Target branch: **`develop`** (default).
- PR title mirrors the branch intent: `Add user authentication`, `Fix login redirect`.

### Description

Every PR must include:

```
## What
One or two sentences on what changed.

## Why
The motivation or ticket reference (e.g. closes #42).

## Notes for reviewer
Anything non-obvious, risky, or worth extra scrutiny.
```

### Before Marking Ready

- [ ] Branch is rebased on latest `develop`
- [ ] CI is green — or any failure is understood and explicitly noted in the description
- [ ] No debug code, commented-out blocks, or stray `console.log` / `print` statements
- [ ] Description is filled out

### Responding to Review Feedback

- **Fix on the same branch** — push new commits, do not open a new PR.
- **Do not rewrite history after reviewers have engaged** — no `rebase` or `push --force` once inline review comments exist.
- If a comment is resolved, mark it resolved. If you disagree, reply with reasoning before closing it.

### Resolving Conflicts Before Merge

When a PR has conflicts, rebase onto `develop` — do not merge `develop` into your branch. This is an expected workflow step; `--force-with-lease` is appropriate here provided no inline review comments have been left that would be displaced.

```bash
git fetch origin
git rebase origin/develop
# resolve conflicts file by file — follow the rules in Handling Problems > Merge / Rebase Conflicts
git push --force-with-lease origin feature/your-branch
```

After pushing, leave a PR comment describing what conflicted and how it was resolved.

→ For the full step-by-step procedure, stop conditions, and escalation rules: `docs/git-conflict-resolution-prompt.md`

### CI Failures

- Fix on the branch and push — never close and reopen the PR.
- If the failure is pre-existing or clearly unrelated to your change, note it explicitly in a PR comment and flag it for the reviewer.

### After Merge

- Delete the remote branch immediately.
- Do not reuse the branch name for future work.

---

## Hard Rules

- **No force-push to `develop` or `main`** under any circumstances.
- **`--force-with-lease` is permitted on feature branches only** — for rebase and conflict resolution, and only before reviewers have left inline comments.
- **No secrets, credentials, or environment files** committed — ever. See *Secret or Credential Accidentally Committed* above.
- If unsure about a destructive operation (`reset`, `rebase` on shared branches, `push --force`): **stop and ask**.