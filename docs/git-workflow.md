# Git Workflow Instructions

> **Audience:** the branch, commit and PR conventions here apply to anyone
> working on `mdv`, including outside contributors. Two sections are maintainer
> procedures that a contributor never runs: *Promoting `develop` → `main`* and
> *PR Merged onto the Wrong Branch*. Two further points differ for a PR opened
> from a **fork**: whether to open it as a draft is the contributor's own call,
> and the "CI is green" item under *Pull Requests > Before Marking Ready* cannot
> be satisfied by the contributor alone — fork workflows do not run until a
> maintainer approves them. See `CONTRIBUTING.md`.

## Core Principles

- **Never commit directly to `main` or `develop`.** All work happens on a
  dedicated branch and lands through a PR — no exception for a one-line or
  documentation-only change. Branch protection with no bypass on either branch
  (#30) is the backstop, and it is live: a direct push to either branch is
  rejected by the server, the maintainer included.
- Keep commits small, atomic, and focused on a single concern.
- Follow the Conventional Commits specification: `<type>[optional scope]: <description>`. Types: `feat:` (MINOR), `fix:` (PATCH), `BREAKING CHANGE` or `!` after type/scope (MAJOR), plus `build:`, `chore:`, `ci:`, `docs:`, `style:`, `refactor:`, `perf:`, `test:`, `hotfix:`. Use imperative mood in descriptions.

### Example Commit Messages

```
feat: add user authentication
fix: resolve login redirect issue
docs: update README with new setup instructions
chore: update dependencies
```

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
>   (Exactly two PRs legitimately target `main`: a promotion — see *Promoting
>   `develop` → `main`* — and a revert of a merge that landed on `main` by
>   mistake, see *PR Merged onto the Wrong Branch*. There is no separate
>   hotfix path despite the `hotfix:` commit type above: an urgent fix branches
>   from `develop` like anything else and reaches `main` by promotion.)

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

# 4. Push and open the PR targeting develop
#    --base is required: the repo's default branch is main, not develop.
#    Drop --draft on a trivial change — see Pull Requests > Opening.
#    Always pass --body-file: without a body argument gh opens an editor,
#    which blocks an autonomous agent, and the draft header block
#    (AGENTS.md §2) would be missing.
git push -u origin feature/my-change
gh pr create --draft --base develop \
  --title "Add concise description of change" \
  --body-file <scratch-file>
```

---

## Keeping Your Branch Up to Date

Rebase onto `develop` before the first commit of any working period, and always before marking a PR ready. Do not let divergence accumulate.

```bash
git fetch origin
git rebase origin/develop
```

**Feature branches only.** Never run this on either of the two `main`-targeting
PRs (*Branch Strategy*): a promotion's head *is* `develop`, so rebasing it
rewrites `develop` — forbidden outright by *Hard Rules* — and a revert onto
`main` must stay based on `main`, or it drags all of `develop` into the revert.

Prefer rebase over merge to keep history linear.

**Solo-workflow rule:** the "do not rewrite history" guard below protects one
thing — a reviewer's inline comments, which rewriting history displaces. So its
trigger is those comments existing, **not** the PR's draft state: rebase freely
for as long as no inline review comment has been left, whether the PR is a
draft, ready-for-review, or not yet opened. Once one exists, stop rewriting
history and follow *Responding to Review Feedback*.

Keying the guard to the draft→ready conversion instead would break both ways on
the three PR kinds that open ready-for-review (*Pull Requests > Opening*): the
conversion never happens, so on one reading the guard never fires at all, and on
the other it fires the instant the PR opens — which would forbid the rebase that
*Resolving Conflicts Before Merge* requires, leaving a conflicted trivial PR with
no legal way to merge.

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

# 3. Open the revert PR against main — ready-for-review, not a draft.
#    --body-file is required: with no body argument gh opens an editor,
#    which blocks an autonomous agent. Write the What/Why/Notes body
#    (see Pull Requests > Description) to a scratch file first.
gh pr create --base main --head fix/revert-merge-onto-main \
  --title "Revert merge onto main" \
  --body-file <scratch-file>

# 4. Once it is merged, verify main is clean — compare main against the state
#    it was in before the bad merge landed, which is that merge's FIRST parent.
#    Empty means the revert restored main exactly.
git fetch origin
git log --oneline -5 origin/main
git diff <merge-commit-sha>^1 origin/main   # expect empty

#    This assumes nothing landed on main after the bad merge — the normal case,
#    since you revert as soon as you notice. If something did (a promotion, say)
#    the diff can never be empty: check instead that the bad merge's own changes
#    are gone, by reading what it brought and confirming none of it remains.
git diff <merge-commit-sha>^1 <merge-commit-sha>   # what the bad merge added

# 5. Re-land the feature on develop — with FRESH commits. See the note below:
#    re-merging the original commits leaves main without the feature forever.
#    Do not go looking for the original branch: this repo deletes head branches
#    on merge and the bad merge did exactly that. Take the commits from the bad
#    merge itself — ^1..^2 is precisely the range it brought in.
git fetch origin && git checkout -b feature/your-branch-redo origin/develop
git cherry-pick <merge-commit-sha>^1..<merge-commit-sha>^2
git log --oneline origin/develop..HEAD      # SHAs must differ from the originals
git push -u origin feature/your-branch-redo
```

> `-m 1` is required for merge commits. It tells git which parent to restore: `1` is the branch merged *into* (`main`), `2` is the branch merged *from*. Without it, git cannot determine which side to revert to.

> **Do not verify step 4 with `git diff origin/develop origin/main`.** `main`
> legitimately lags `develop` between promotions, so that diff is non-empty in
> normal operation and says nothing about whether the revert worked. An empty
> `main`↔`develop` diff is the success condition for a **promotion** (see
> *Promoting `develop` → `main`*), not for a revert.

> **Why step 5's commits have to be new.** The originals are already in `main`'s
> history through the bad merge, and the revert undid their *content* without
> removing the *commits*. Merge them into `develop` unchanged and the next
> promotion brings nothing over: they are already in the promotion's merge base,
> so git sees no change to apply. `main` ends up permanently without the
> feature, and the promotion's own `git diff origin/main origin/develop` check
> comes back non-empty with no obvious cause. Cherry-picking re-times each
> commit and so gives it a SHA `main` has never seen, which the promotion then
> carries normally — that is what step 5's `git log` check confirms. The redo
> branch needs its own PR into `develop`; the original PR stays closed.

### Branched from Wrong Base (e.g. from `main` instead of `develop`)

Usually the default-branch trap described under *Branch Strategy*: a fresh clone
lands on `main`, and branching straight from it bases the work there.

**This section does not apply to a revert of a merge that reached `main` by
mistake.** That branch is based on `main` deliberately — see *PR Merged onto the
Wrong Branch* — and running the `rebase --onto` below on it would destroy the
revert it carries and drag all of `develop` onto a `main`-based branch. Being
based on `main` is only a mistake on a branch that was headed for `develop`.

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

- **Open a draft PR immediately after the first push** — on non-trivial feature
  work, do not wait until the work is done. This makes the branch visible,
  triggers CI early, and signals work-in-progress.
- **Three kinds of PR skip the draft stage** and open ready-for-review: a
  trivial change (`AGENTS.md` §1), a promotion PR (*Promoting `develop` →
  `main`*), and a revert of a merge that landed on `main` by mistake (*PR Merged
  onto the Wrong Branch*). None has a work-in-progress period to signal.
- **A PR from a fork is the contributor's own call.** Nothing here obliges an
  outside contributor to open a draft first; the three kinds above are the ones
  that skip the stage by rule rather than by choice. See `CONTRIBUTING.md`.
- Target branch: **`develop`** — pass `--base develop` explicitly. It is not the
  tool's default; `gh pr create` would open against `main`. See *Branch Strategy*.
- PR title mirrors the branch intent: `Add user authentication`, `Fix login redirect`.

### Description

Every PR body must include the following sections — except a promotion PR, which
uses the body given in *Promoting `develop` → `main`* instead. When the PR is
opened as a draft, also include the draft header block defined in AGENTS.md §2
(`Plan: / Source: / Status: WIP`) above these sections — the two are
complementary: the header carries machine-readable status and the plan pointer,
the sections below carry the human-readable summary.

```
## What
One or two sentences on what changed.

## Why
The motivation or ticket reference. Do **not** write `closes #<N>` / `Closes #<N>`
while the PR is a draft — a closing keyword auto-closes the issue on merge before
the work is verified. Reference the issue as `ref #<N>` for as long as the PR is
a draft, and add `Closes #<N>` when it is converted to ready-for-review — or from
the start on a PR that has no draft stage (see AGENTS.md §2).

## Notes for reviewer
Anything non-obvious, risky, or worth extra scrutiny.
```

### Before Marking Ready

Run this when converting a draft to ready-for-review — or, on a PR that has no
draft stage (a trivial change, a promotion, a revert onto `main`), before
merging it. **The CI item is the exception to that timing:** no workflow runs
until the PR exists, since CI triggers on `pull_request` and on pushes to
`main`/`develop` only — never on a push to a feature branch. So on a no-draft
PR, check every other item before you open it, and the CI item once it is open.

- [ ] Branch is rebased on latest `develop` — does not apply to the two
      `main`-targeting PRs: a promotion's head *is* `develop`, and a revert onto
      `main` must stay based on `main`
- [ ] No debug code, commented-out blocks, or stray `console.log` / `print` statements
- [ ] Description is filled out
- [ ] CI is green — or any failure is understood and explicitly noted in the
      description. Only observable once the PR is open, and on a PR from a fork
      only after a maintainer approves the workflow run, which is not the
      contributor's to do.

### Responding to Review Feedback

- **Fix on the same branch** — push new commits, do not open a new PR.
  **Feature branches only.** A promotion PR's head branch *is* `develop`, so
  pushing a fix to it is a direct commit to `develop`, which *Core Principles*
  forbids: land the fix as an ordinary PR into `develop` and the promotion PR
  picks it up on its own. A revert PR takes fixes on its own `main`-based branch.
- **Do not rewrite history once review has begun** — no `rebase` or `push --force` once any inline review comment exists. Before that point rebasing is fine regardless of the PR's draft state — see the *Solo-workflow rule* under *Keeping Your Branch Up to Date*.
- If a comment is resolved, mark it resolved. If you disagree, reply with reasoning before closing it.

### Resolving Conflicts Before Merge

When a PR has conflicts, rebase onto `develop` — do not merge `develop` into your branch. This is an expected workflow step; `--force-with-lease` is appropriate here provided no inline review comments have been left that would be displaced.

**Feature branches only**, for the reasons given under *Keeping Your Branch Up to
Date* — the force-push below would land on `develop` itself if run on a
promotion PR. The two `main`-targeting PRs resolve conflicts differently:

- A **promotion PR should not conflict at all.** If it does, `main` has diverged
  and the precondition in *Promoting `develop` → `main`* has already failed —
  that needs re-planning, not conflict resolution. Stop and escalate.
- A **revert PR** rebases onto `origin/main`, never `origin/develop`.

```bash
git fetch origin
git rebase origin/develop
# resolve conflicts file by file — follow the rules in Handling Problems > Merge / Rebase Conflicts
git push --force-with-lease origin feature/your-branch
```

After pushing, leave a PR comment describing what conflicted and how it was resolved.

→ For the full step-by-step procedure, stop conditions, and escalation rules: `docs/git-conflict-resolution.md`

### CI Failures

- Fix on the branch and push — never close and reopen the PR. The **feature
  branches only** qualifier under *Responding to Review Feedback* applies here
  too: a promotion PR's head is `develop`, which takes no direct pushes.
- If the failure is pre-existing or clearly unrelated to your change, note it explicitly in a PR comment and flag it for the reviewer.

### After Merge

- The remote branch is deleted automatically when the PR merges. If it survives,
  delete it manually — the fallback, not the normal path. (Promotion PRs are the
  case to watch, since their head branch is `develop` itself — see *Promoting
  `develop` → `main`*.)
- Do not reuse the branch name for future work. **Feature branches only** — the
  two `main`-targeting PRs reuse theirs by construction: a promotion's head is
  always `develop`, and the revert recipe names its branch
  `fix/revert-merge-onto-main` every time it runs.

---

## Promoting `develop` → `main`

`main` is the public front page of the repo; `develop` is where work
integrates. Promotion is the routine operation that brings `main` up to date —
not a release ceremony and not a one-off. `main` falls behind on every merge
into `develop`, so this runs repeatedly. The whole procedure is below.

### Preconditions

```bash
git fetch origin

# 1. The payload — expect only the work you intend to promote.
#    Empty? main is already current and there is nothing to promote — stop here.
#    `gh pr create` would fail with "No commits between main and develop".
git diff origin/main origin/develop
git log --oneline origin/main..origin/develop

# 2. main must carry no unique work of its own
#    Expect empty — with one legitimate exception, see below.
git log --oneline --no-merges origin/develop..origin/main
```

The second check is `--no-merges` deliberately. `main` legitimately accumulates
one merge commit per promotion that `develop` never sees — that is how this
model works, and it is **not** divergence. A **non-merge** commit on `main` is
normally divergence: something landed there directly. That needs re-planning,
not force-merging. Stop and escalate.

**Repairing a wrong-branch merge leaves permanent residue here, and it is not
divergence.** Once *PR Merged onto the Wrong Branch* has run, `main` carries
non-merge commits `develop` will never have: every original commit of the
mis-merged feature — they reached `main` through the bad merge and were never on
`develop` — plus the revert that neutralised them. The recovery re-lands that
work on `develop` under *new* SHAs, so the originals stay on `main` forever.
This check is therefore non-empty on **every** promotion from then on, by a
count that grows with the size of the feature that was mis-merged.

So treat this check as a heuristic, not the authority. Confirm that everything
it lists is either that revert or one of the mis-merged originals, and proceed
if so — **precondition 1's content diff is what actually decides whether `main`
has diverged.** Anything in the list that is neither is real divergence: stop
and escalate.

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

# 2. develop still exists — ask the remote, not your cache.
#    `git branch -r` reads remote-tracking refs, which a plain `git fetch`
#    does NOT prune, so it would still list origin/develop after GitHub
#    deleted it. This must query the remote directly.
git ls-remote --heads origin develop       # expect one line; empty means deleted
```

Check the **diff**, not the log. `main` now carries a merge commit that
`develop` does not; that is expected, and is not divergence. Merging `main`
back into `develop` is **not** required.

> **This is why "Require branches to be up to date before merging" is
> deliberately OFF** in the branch protection ruleset (#30), despite being
> switched on for most repositories. That setting requires a PR's head branch to
> contain the base branch's tip. After every promotion `develop` is behind `main`
> by exactly the merge commit above — legitimately, per the paragraph you just
> read — so with the setting on, **the next promotion PR would be blocked**.
> GitHub's only offered remedy is *Update branch*, which back-merges `main` into
> `develop`: precisely what this section says is not required, and it would put
> promotion merge commits onto `develop` permanently, breaking the two-branch
> model on every promotion thereafter.
>
> Little is given up by turning it off. Checks on a `pull_request` event already
> run against `refs/pull/N/merge` — a simulated merge of head into base — so the
> integration testing that setting exists to guarantee is already happening. It
> closes only the narrow race where the base moves between a check completing and
> the merge button being clicked.
>
> **Do not switch it on** without re-reading this section first.

The second check exists because this repo deletes head branches automatically
on merge, and a promotion PR's head branch is `develop` itself. GitHub skips
protected and default branches — `main` is the default — so now that branch
protection covers `develop` (#30), **this hazard has lapsed permanently**:
`develop` is protected and will no longer be auto-deleted.

The check and the recovery below are kept anyway. The hazard was real and fired
exactly once, on the #41 promotion, which ran before the ruleset existed —
`develop` was deleted and restored by hand. Keeping the check costs one command
and would catch any future change to the protection settings. If `develop` is
ever missing, restore it:

```bash
# Restore from the promotion merge itself, not from whatever your local
# develop happens to be — a stale clone would recreate an older tip.
# Parent 2 of that merge IS the exact tip develop had when it merged.
git fetch origin '+refs/heads/main:refs/remotes/origin/main'

# Name the promotion merge explicitly. Do NOT shorten this to `origin/main^2`:
# main's tip is not always the promotion merge — a revert PR (PR Merged onto
# the Wrong Branch) also lands there as a merge commit — and on any other merge
# `^2` resolves to THAT merge's second parent and recreates develop at the wrong
# commit, silently and with exit status 0.
git log --merges --oneline origin/main | head -5    # identify the promotion merge
PROMO=<sha-of-the-promotion-merge>

# --verify is what makes a wrong SHA fail loudly. Without it the command
# substitution is never empty even on failure: `git rev-parse` prints its own
# argument back on stdout, so the push runs with a garbage refspec instead.
git rev-parse --verify "$PROMO^2"
git push origin "$PROMO^2":refs/heads/develop
git fetch origin && git checkout develop && git reset --hard origin/develop
```

GitHub also offers a **Restore branch** button on the merged PR, which restores
the same commit. Nothing is ever lost either way: once the promotion has merged,
`develop`'s tip is reachable from `main` as parent 2 of the merge commit, so
deletion costs a ref, never history.

### Cadence

A judgement call, not a rule. There is no need to promote after every merge
into `develop`, but `main` is what a visitor to the public repo sees — let it
drift and the front page goes stale.

---

## Hard Rules

- **No force-push to `develop` or `main`** under any circumstances.
- **`--force-with-lease` is permitted on any branch except `develop` and `main`** — feature branches and the `main`-based revert branch of *PR Merged onto the Wrong Branch* — for rebase and conflict resolution, and only before any inline review comment has been left.
- **No secrets, credentials, or environment files** committed — ever. See *Secret or Credential Accidentally Committed* above.
- If unsure about a destructive operation (`reset`, `rebase` on shared branches, `push --force`): **stop and ask**.
