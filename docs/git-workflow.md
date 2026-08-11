# Git Workflow Instructions

## Core Principles

- **Never commit directly to `main` or `develop`.** All work happens on a
  dedicated branch and lands through a PR — no exception for a one-line or
  documentation-only change. Enforced by branch protection with no bypass on
  either branch (#30).
- Keep commits small, atomic, and focused on a single concern.
- Follow the Conventional Commits specification: `<type>[optional scope]: <description>`. Types: `feat:` (MINOR), `fix:` (PATCH), `BREAKING CHANGE` or `!` after type/scope (MAJOR), plus `build:`, `chore:`, `ci:`, `docs:`, `style:`, `refactor:`, `perf:`, `test:`, `hotfix:`. Use imperative mood in descriptions.

### Example Commit Messages

```feat: add user authentication```
```fix: resolve login redirect issue```
```docs: update README with new setup instructions```
```chore: update dependencies```

When a commit completes a plan task, append the task and issue reference inline
in the subject, after the description: `feat: add TOC overlay (task 3, ref #42)`.
For non-issue-driven work, omit the parenthetical entirely and use a plain
Conventional Commit: `feat: add TOC overlay`. See AGENTS.md §1 step 4 for the
per-task commit cadence.

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

> **The repo's default branch is `main`, not `develop`.** Tooling follows the
> default, so it points at the wrong branch unless you say otherwise. Two
> consequences, both easy to hit:
>
> - A fresh `git clone` checks out `main`. Branching from there without the
>   `git checkout develop` step above bases your work on the wrong branch — see
>   *Branched from Wrong Base* to recover.
> - `gh pr create` targets the default branch when `--base` is omitted, so
>   **every PR for feature work needs `--base develop` passed explicitly.**
>   (The one PR that legitimately targets `main` is a promotion — see
>   *Promoting `develop` → `main`*.)

---

## Daily Workflow

```bash
# 1. Start from an up-to-date develop
git checkout develop && git pull origin develop

# 2. Create your branch
git checkout -b feature/my-change

# 3. Work, then stage and commit
# Human: stage hunks interactively to review what you commit.
git add -p
# Autonomous agent: interactive staging is not possible. Stage explicit,
# intended paths instead — never `git add .` or `git add -A` blindly.
git add path/to/changed_file.py path/to/other_file.py
git commit -m "feat: add concise description of change"

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

**Solo-workflow rule:** rebase freely while the PR is still a draft — there are
no reviewers whose inline comments could be displaced, so the "do not rewrite
history" guard below does not yet apply. Once the PR is converted to
ready-for-review (or any reviewer leaves an inline comment), stop rewriting
history and follow *Responding to Review Feedback*.

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

**This covers a merge that should never have landed on `main`** — typically a
feature PR opened against the wrong base. Promoting `develop` into `main` is a
legitimate merge and is *not* this case; see *Promoting `develop` → `main`*.
Applying the recipe below to a promotion would revert the entire project back
off `main`.

Always use `git revert -m 1` — never `reset --hard`. By the time a merge lands on `main`, CI/CD pipelines and other collaborators have likely already seen it. Rewriting history is unsafe; a revert commit is the correct response.

The revert itself goes through a PR like any other change — *Core Principles*
forbids pushing directly to `main`.

```bash
# 1. Identify the bad merge commit SHA
git log --oneline -5 origin/main

# 2. Revert it on a branch — -m 1 restores main's side (parent 1), discarding
#    the feature branch changes
git checkout main
git pull origin main
git checkout -b fix/revert-merge-onto-main
git revert -m 1 <merge-commit-sha>
git push -u origin fix/revert-merge-onto-main

# 3. Open the revert PR against main
gh pr create --base main --head fix/revert-merge-onto-main \
  --title "Revert merge onto main"

# 4. Once it is merged, verify main is clean
git fetch origin
git log --oneline -5 origin/main
git diff origin/develop origin/main

# 5. The feature branch is untouched — open a new PR targeting develop
git checkout feature/your-branch
```

> `-m 1` is required for merge commits. It tells git which parent to restore: `1` is the branch merged *into* (`main`), `2` is the branch merged *from*. Without it, git cannot determine which side to revert to.

### Branched from Wrong Base (e.g. from `main` instead of `develop`)

Usually the default-branch trap described under *Branch Strategy*: a fresh clone
lands on `main`, and branching straight from it bases the work there.

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
- Target branch: **`develop`** — pass `--base develop` explicitly. It is not the
  tool's default; `gh pr create` would open against `main`. See *Branch Strategy*.
- PR title mirrors the branch intent: `Add user authentication`, `Fix login redirect`.

### Description

Every PR body must include the following sections. When the PR is opened as a
draft, also include the draft header block defined in AGENTS.md §2
(`Plan: / Source: / Status: WIP`) above these sections — the two are
complementary: the header carries machine-readable status and the plan pointer,
the sections below carry the human-readable summary.

```
## What
One or two sentences on what changed.

## Why
The motivation or ticket reference. Do **not** write `closes #<N>` / `Closes #<N>`
while the PR is a draft — a closing keyword auto-closes the issue on merge before
the work is verified. Reference the issue as `ref #<N>` until the PR is converted
to ready-for-review, then add `Closes #<N>` (see AGENTS.md §2).

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
- **Do not rewrite history once review has begun** — no `rebase` or `push --force` after the PR is converted to ready-for-review or any inline review comment exists, whichever comes first. Before that point (draft stage), rebasing is fine — see *Keeping Your Branch Up to Date*.
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

→ For the full step-by-step procedure, stop conditions, and escalation rules: `docs/git-conflict-resolution.md`

### CI Failures

- Fix on the branch and push — never close and reopen the PR.
- If the failure is pre-existing or clearly unrelated to your change, note it explicitly in a PR comment and flag it for the reviewer.

### After Merge

- The remote branch is deleted automatically when the PR merges. If it survives,
  delete it manually — the fallback, not the normal path. (Promotion PRs are the
  case to watch, since their head branch is `develop` itself — see *Promoting
  `develop` → `main`*.)
- Do not reuse the branch name for future work.

---

## Promoting `develop` → `main`

`main` is the public front page of the repo; `develop` is where work
integrates. Promotion is the routine operation that brings `main` up to date —
not a release ceremony and not a one-off. `main` falls behind on every merge
into `develop`, so this runs repeatedly. The whole procedure is below.

### Preconditions

```bash
git fetch origin

# 1. The payload — expect only the work you intend to promote
git diff origin/main origin/develop
git log --oneline origin/main..origin/develop

# 2. main must carry no unique work of its own
git log --oneline --no-merges origin/develop..origin/main   # expect empty
```

The second check is `--no-merges` deliberately. `main` legitimately accumulates
one merge commit per promotion that `develop` never sees — that is how this
model works, and it is **not** divergence. A **non-merge** commit on `main` is
divergence: something landed there directly. That needs re-planning, not
force-merging. Stop and escalate.

### Open the PR

Promotion goes through a PR — never a direct push. *Core Principles* forbids
committing directly to either branch (enforced by branch protection on both
branches, #30), and the PR is also what gets the promotion tested: the CI and
audit workflows both run on pull requests targeting `main`.

```bash
# write the body to a scratch file first, then:
gh pr create --base main --head develop \
  --title "Promote develop to main" \
  --body-file <scratch-file>
```

PR body — copy this as-is:

```
Brings `main` up to date with `develop`.

**Merge with "Create a merge commit" — not squash, not rebase.** Squashing
collapses the promoted history into a single commit on `main`. Rebasing
replays it under new SHAs. Both permanently diverge `main` from `develop` and
break every future promotion.
```

That paragraph is the safeguard for whoever clicks the merge button, not
filler. Reproduce it in every promotion PR.

### Merge method: "Create a merge commit"

**Never squash. Never rebase.** All three merge methods are enabled on this
repo and GitHub pre-selects whichever was used last, so the dropdown sits one
absent-minded click away from an irreversible mistake.

- **Squash** collapses the entire promoted history into one commit on `main` —
  literally the history curation #20 considered and rejected — and diverges
  `main` from `develop` permanently.
- **Rebase** replays the same commits onto `main` under new SHAs, duplicating
  the history under different identities. Same divergence, different route.
- **Merge commit** preserves every commit as-is, and is the only option
  consistent with promoting `develop` as it stands.

The divergence is not cosmetic: once `main` holds commits `develop` cannot
recognise, the precondition above fails on every subsequent promotion and each
one turns into a conflict-resolution exercise.

Merging the PR is the maintainer's action, as with any PR in this repo.

### After the merge

```bash
git fetch origin

# 1. The two branches hold identical trees
git diff origin/main origin/develop        # expect empty

# 2. develop still exists
git branch -r | grep origin/develop
```

Check the **diff**, not the log. `main` now carries a merge commit that
`develop` does not; that is expected, and is not divergence. Merging `main`
back into `develop` is **not** required.

The second check exists because this repo deletes head branches automatically
on merge, and a promotion PR's head branch is `develop` itself. GitHub skips
protected and default branches — `main` is the default — so once branch
protection covers `develop` (#30) the hazard lapses permanently. Until then, if
`develop` is missing, restore it:

```bash
git push origin develop        # from an up-to-date local develop
```

GitHub also offers a **Restore branch** button on the merged PR.

### Cadence

A judgement call, not a rule. There is no need to promote after every merge
into `develop`, but `main` is what a visitor to the public repo sees — let it
drift and the front page goes stale.

---

## Hard Rules

- **No force-push to `develop` or `main`** under any circumstances.
- **`--force-with-lease` is permitted on feature branches only** — for rebase and conflict resolution, and only before reviewers have left inline comments.
- **No secrets, credentials, or environment files** committed — ever. See *Secret or Credential Accidentally Committed* above.
- If unsure about a destructive operation (`reset`, `rebase` on shared branches, `push --force`): **stop and ask**.
