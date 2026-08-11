# PLAN: Align workflow documentation with branch protection

_Branch:_ `chore/issue-35-workflow-docs`
_Date:_ 2026-08-11
_Status:_ READY
_Source:_ #35

---

## Problem

The three instruction documents (`AGENTS.md`, `docs/git-workflow.md`,
`CONTRIBUTING.md`) are out of step with the repository as it now exists. Two
changes must land together, followed by a consistency review across all three.

**Part A — the `develop` → `main` promotion procedure is undocumented.**
`docs/git-workflow.md` describes the two-branch model but never says how work
gets from `develop` to `main`. The procedure and its safeguards exist in exactly
one place: step R1 of `docs/plans/PLAN-public-release-readiness.md`, a merged
plan file — a historical record nobody consults before a git operation, while
`AGENTS.md` §2 directs the reader to `docs/git-workflow.md`, where no relevant
section exists. Promotion is now a recurring operation (`main` is the public
front page and falls behind on every merge into `develop`), and the expensive
failure mode — picking "Squash and merge" from the dropdown by habit — is one
click away, since all three merge methods are enabled on this repo.

**Part B — `AGENTS.md`'s trivial-change direct-commit exemption becomes
unworkable under #30.** #30 protects `main` and `develop` with required pull
requests, zero required approvals and **no admin bypass**. `AGENTS.md:54–56`
says trivial changes commit directly to the current branch with no PR; under
protection that instruction fails at the push with no documented recovery. It
also already contradicts `docs/git-workflow.md:5` ("Never commit directly to
`main` or `develop`") — a contradiction that has stood for the life of the repo.
Only the direct-commit exemption is dropped; the trivial/non-trivial distinction
and its plan-file, label and test-loop exemptions stay, because protection
changes the *path a commit takes to a protected branch*, not how much process a
change deserves.

**Part C — consistency review.** The bar is that an agent can implement from
these documents without guessing. Findings are batched and surfaced to the
maintainer via `needs-decision` before the documents are finalised, not resolved
unilaterally.

Documentation only — no code, no CI, no change to the binary.

## Out of scope

- Creating the branch-protection ruleset (#30) — this issue documents the
  contract, #30 enforces it.
- The `develop` → `main` promotion PR itself. This plan writes the procedure;
  running it is the maintainer's action after this PR merges, and it is the
  first real exercise of the new section.
- Any change to the trivial/non-trivial *definition*, or to the plan-file,
  label-workflow and test-loop exemptions for trivial changes.
- The Actions `allowed_actions` allowlist (#36) and anything else split out of
  #30.
- Identifying or disabling the editor-side Markdown formatter (see Risks) —
  this plan only guards against it, it does not hunt it down.
- `README.md`, `docs/architecture.md`, `docs/mdv-build-plan.md`,
  `docs/git-conflict-resolution.md` and `docs/plans/*` are not edited, except
  for the Finish-checklist `README.md` check and cross-reference verification
  in task 8.

## Impact assessment

| File | Change |
|---|---|
| `docs/git-workflow.md` | New promotion section (A1); reframe `### PR Merged onto the Wrong Branch` at `:143` (A2); default-branch warnings at `:34–43`, `:220` (A3); consistency of `:5`, `:280`, `:287` (B4) |
| `AGENTS.md` | Rewrite trivial-change block at `:51–61` (B1); reconcile step 3 at `:94–96` (B2); branch naming for trivial changes (B3) |
| `CONTRIBUTING.md` | `:15` target-branch wording; fork-PR consequences (B5) |
| `docs/plans/PLAN-workflow-docs-alignment.md` | This plan |

**Verified on the ground, 2026-08-11** (re-check at implementation time if this
sits unworked):

- `git log origin/develop..origin/main` returns **one commit** — the `#32`
  promotion merge `549da73`. It is **not** empty, so the precondition as worded
  in the issue would fail every future promotion. See Open questions.
- `git log origin/main..origin/develop` returns 2 commits (`22daf27`, `8d5a29f`)
  — the pending promotion payload.
- `main` is **not protected** (`gh api …/branches/main/protection` → 404
  "Branch not protected"), confirming #30 has not landed. Sequencing holds.
- All three merge methods are still enabled: `mergeCommitAllowed`,
  `squashMergeAllowed`, `rebaseMergeAllowed` all `true`. The dropdown hazard A1
  guards against is live.
- `defaultBranchRef` is `main` and `origin/HEAD → origin/main`, so a bare
  `gh pr create` targets `main`. A3's finding is confirmed, not theoretical.
- **`deleteBranchOnMerge` is now `true`** (enabled 2026-08-11 under #30's
  already-applied settings work). The promotion PR's *head* branch is `develop`,
  and `develop` is currently unprotected — see Risks.
- `#33` (`22daf27`) is merged into `develop`. Prerequisite satisfied.
- No repo-side formatter config exists (`.editorconfig`, `.prettierrc`,
  `.vscode/` all absent) and `.claude/settings.local.json` defines no hooks —
  the formatter that damaged `PLAN-public-release-readiness.md` is editor-side
  and outside this repo's control.

**Conventions for the implementer** — these apply to every task below and exist
to remove guesswork that would otherwise be resolved differently task by task:

- **Every line number in this plan is as of `develop@22daf27`.** Task 1 inserts
  a large section into `docs/git-workflow.md`, so every later line reference in
  that file shifts the moment it lands. **Locate targets by heading or by the
  quoted text given, never by line number.** Line numbers here are provenance,
  not addresses.
- **Do not renumber or retitle `AGENTS.md` §1–§5, or the numbered steps inside
  §1.** `docs/git-workflow.md:19` cites "§1 **step 4**" specifically, `:226` and
  `:239` cite "§2", and `docs/plans/PLAN-template.md:6,68` cites "§5" — all from
  outside the three documents this plan may edit. Editing prose inside a section
  or a step is fine; changing a section number, a step number, or a heading is a
  C2 escalation, not an implementer's call, because it silently invalidates
  references this plan does not permit fixing. Task 5's relocation is the one
  place this constraint bites — see that task.
- **Enforcement framing:** #30 has not landed when this PR merges, so these
  documents must not assert branch protection as a live fact. Write each rule as
  a rule in the present tense ("never commit directly to `main` or `develop`"),
  and where enforcement is worth naming, attribute it — "enforced by branch
  protection on both branches (#30)" — without a claim about when it took
  effect. The rules stand on their own; protection is the backstop, not the
  reason. See Open question 6.
- **Push before stopping at task 10.** That task ends the session pending a
  maintainer decision, so everything through task 9 must be committed and pushed
  first (`AGENTS.md` §1 step 4).

**Risks**

- **Auto-delete of `develop` on the promotion merge.** With
  `deleteBranchOnMerge: true`, merging a PR whose head is `develop` invites
  GitHub to delete the head branch. GitHub skips protected and default branches
  — `main` is the default, but `develop` is neither protected (until #30) nor
  default, so the very next promotion is the exposed one. Recovery is cheap
  (GitHub offers "Restore branch"; a local `git push origin develop` also
  restores it), but an unwritten hazard on the first exercise of a
  freshly-written procedure is exactly what this issue exists to prevent. The
  promotion section must carry a post-merge check for it. Once #30 protects
  `develop`, the hazard disappears permanently.
- **The editor-side Markdown formatter.** It rewrote
  `PLAN-public-release-readiness.md` — collapsing fenced code blocks into inline
  spans and HTML-escaping blockquotes — and has not been identified. All three
  documents in this plan use fenced blocks and blockquotes heavily, and damage
  to a live instruction document is materially worse than to a completed plan
  file. Task 9 adds an explicit integrity check, and every commit's diff must be
  read before staging rather than `git add`-ed on faith.
- **Part C is not a formality.** Part B changes the project's working contract,
  and the contradiction it fixes sat undetected for the life of the repo. Task 8
  should be expected to produce findings; if it produces none, it was not run
  properly.
- **No test signal.** Documentation changes cannot be verified by the test
  suite. The only real verification is the C1 read-through and the maintainer
  round-trip in C2, which is why they are tasks rather than checklist items.

## Open questions

Findings that surfaced during assessment. None block starting implementation —
each has a stated assumption — but all six are folded into the C2 batch in
task 10 so the maintainer rules on them alongside the C1 findings.

1. **The promotion precondition cannot be "`git log origin/develop..origin/main`
   is empty"** — it already isn't, and never will be again. The `#32` merge
   commit lives on `main` by design, and each future promotion adds another.
   → [ASSUMPTION: write the precondition content-first — `git diff
   origin/main origin/develop` shows only the intended payload, and
   `git log --no-merges origin/develop..origin/main` is empty (no unique *work*
   on `main`). Non-merge commits on `main` mean real divergence and trigger the
   re-plan escalation the issue asks for; promotion merge commits do not.]

2. **Auto-delete of `develop` on promotion** (see Risks). → [ASSUMPTION:
   document it in the promotion section as a post-merge verification step with
   the restore command, rather than recommending a repo-settings change.
   `deleteBranchOnMerge: true` is correct for feature branches, which is the
   overwhelmingly common case, and #30 removes the exposure for `develop`
   shortly afterwards.]

3. **Branch naming for trivial changes (B3).** They have no plan file and so no
   assigned branch name. → [ASSUMPTION: point at `docs/git-workflow.md`'s
   existing convention (`<type>/<short-hyphenated-name>`, ticket number appended
   when one exists) rather than inventing a second rule in `AGENTS.md`.]

4. **`docs/git-workflow.md:280` "Delete the remote branch immediately" is now
   automatic** (`deleteBranchOnMerge: true`). → [ASSUMPTION: reword to note
   GitHub does this on merge and that the manual step is only a fallback —
   folded into task 6, flagged in the C2 batch.]

5. **Does a trivial-change PR follow the §2 draft-PR ritual?** This is the
   largest gap the issue text leaves open, and it is unanswerable from the
   current documents. `AGENTS.md` §2 requires a draft PR after the first push
   carrying a `Plan: / Source: / Status: WIP` header block — but a trivial
   change has no plan file, so the `Plan:` line has nothing to point at, and a
   typo fix does not benefit from a WIP stage. Equally unstated: whether the
   trivial path now runs the §5 Finish step (`/skill:adversarial-review`), which
   it has always been outside of. → [ASSUMPTION: the trivial path opens a
   normal ready-for-review PR — no draft stage, no §2 header block, no
   adversarial review — and task 4 enumerates the skips explicitly rather than
   leaving them to inference. The point of Part B is that protection changes the
   *path*, not the *process*; inheriting the draft ritual would smuggle process
   in through the back door.]

6. **How should the documents refer to branch protection that has not landed
   yet?** #35 merges before #30 by design, so any sentence asserting protection
   as live is false for the window between them. → [ASSUMPTION: rules are stated
   unconditionally in the present tense and protection is named as an attributed
   backstop, never as a dated fact — see *Conventions for the implementer*. This
   also means the documents need no follow-up edit when #30 lands.]

---

## Tasks

### Implementation

- [x] 0. Create branch `chore/issue-35-workflow-docs` from develop following docs/git-workflow.md
      Commit this plan as the first commit with `_Status:_` flipped `DRAFT` →
      `APPROVED`, push immediately, and open the draft PR against `develop` with
      the `AGENTS.md` §2 header block. Use `ref #35`, **not** `Closes #35`, while
      it is a draft.
      **Task 1 is the "first code task"** for the purposes of `AGENTS.md` §1
      step 4 even though this branch touches no code: add `in progress` to #35
      when starting it, and flip `_Status:_` to `IN PROGRESS` in the same commit.

**Part A — promotion procedure**

- [x] 1. **Add a `## Promoting develop → main` section to `docs/git-workflow.md`**,
      after *Pull Requests* and before *Hard Rules*. Carry over R1/R2 of
      `docs/plans/PLAN-public-release-readiness.md:446–491`, generalised from a
      one-off release step into a repeatable procedure. It must state:
      - promotion goes through a PR (`gh pr create --base main --head develop`),
        never a direct push — `docs/git-workflow.md:5` forbids the push, and the
        PR is what gets the promotion tested by CI (both workflows already
        trigger on PRs targeting `main`);
      - **merge method is "Create a merge commit" — never squash, never
        rebase**, with the reasoning kept, not compressed away: all three
        buttons are enabled, GitHub remembers the last method used, squash
        collapses the whole history into one commit on `main` (the curation #20
        rejected), and both squash and rebase permanently diverge `main` from
        `develop`, breaking every future promotion;
      - the PR body must restate the merge-method instruction — it is the
        safeguard for whoever clicks the button, not filler. **Include a
        copy-pasteable PR-body template in a fenced block**, as R1 did, so the
        safeguard is reproduced rather than paraphrased from memory each time;
      - **precondition:** per Open question 1, `git diff origin/main
        origin/develop` shows only the intended payload and
        `git log --no-merges origin/develop..origin/main` is empty. Anything
        else means `main` has diverged and the situation needs re-planning, not
        force-merging;
      - **post-merge verification:** `git diff origin/main origin/develop` is
        empty. Check the *diff*, not the log — `main` legitimately carries one
        merge commit per promotion that `develop` does not, and that is not
        divergence. Merging `main` back into `develop` is not required;
      - **post-merge check that `develop` still exists** and how to restore it
        (Open question 2), noting the hazard lapses once `develop` is protected;
      - **cadence is a judgement call, not a rule** — no need to promote
        per-PR, but `main` is what a visitor to the public repo sees, so
        letting it drift means the front page goes stale.
- [x] 2. **Reframe `### PR Merged onto the Wrong Branch`** (`docs/git-workflow.md:143`).
      It is currently the document's only discussion of a merge landing on
      `main` and treats that as always a mistake; applying its `git revert -m 1`
      recipe to a legitimate promotion would revert the entire project off
      `main`. Add the distinguishing sentence and a pointer to the new
      promotion section.
      **Beyond A2's literal ask, flagged deliberately:** the recipe's
      `git checkout main` … `git push origin main` steps are direct pushes to a
      branch that `:5` forbids and that protection rejects, so the recipe is broken independently
      of the promotion question and repairing it (route the revert through a PR)
      is in scope for this task. Bound the repair to the push mechanics — do not
      rewrite the `-m 1` explanation, which is correct and still needed.
- [x] 3. **Record the default-branch hazard** (A3). `origin/HEAD` resolves to
      `main`, so a bare `gh pr create` opens against `main` and bypasses
      `develop`: `--base develop` is required on every feature PR. Fix
      `docs/git-workflow.md:220`, which says the target branch is "`develop`
      (default)" — it is not the tool's default and the parenthetical is
      actively misleading. Same root cause for a fresh clone landing on `main`.
      **Write the warning once**, under *Branch Strategy* where the branching
      commands live, and reference it from *Pull Requests > Opening* and from
      the existing recovery under *Branched from Wrong Base* — three copies of
      the same caveat is how the documents drift out of agreement in the first
      place.

**Part B — trivial-change contract**

- [x] 4. **Rewrite `AGENTS.md:51–61`** (B1). Keep the definition of trivial
      verbatim (zero logic impact AND limited in scope; when in doubt, treat as
      non-trivial). Replace "commit directly to whichever branch is currently
      checked out. No plan file, no new branch, no PR required" with: branch
      from `develop` and open a PR, but skip the plan file, the label workflow
      and the full test loop. Keep the existing rule that a trivial change
      touching a code file must pass `cargo fmt --check` and
      `cargo clippy --all-targets -- -D warnings`, and that non-code changes
      skip checks entirely. State the resulting cost explicitly — branch →
      commit → PR → merge, nothing else — and say why (no bypass exists on
      either protected branch), so the rule is not read as gratuitous ceremony
      and quietly re-litigated later.
      **Enumerate the trivial path exhaustively rather than by inference** —
      what it *keeps*: a branch off `develop`, a Conventional Commit, a PR into
      `develop`, and `cargo fmt --check` + `cargo clippy --all-targets -- -D
      warnings` when a code file is touched. What it *skips*: the plan file, the
      §5 label workflow, the §3 test loop, the §2 draft-PR stage and its
      `Plan: / Source: / Status:` header block, and the §1 step 5 Finish step
      (`/skill:adversarial-review`). The last two are Open question 5 and are
      the whole reason this must be a list and not a sentence: a reader who
      infers instead of reading will pull the entire non-trivial ritual back in
      through §2, which defeats the change.
- [x] 5. **Reconcile `AGENTS.md:94–96` and cover trivial-change branch naming**
      (B2 + B3). Step 3's "This is mandatory — there are no exceptions based on
      scope, triviality, or whether a commit is imminent" was written to close
      the loophole task 4 deletes.
      **Recommended resolution, so this is not left to the implementer's
      judgement:** keep the sentence — after task 4 it is no longer defending
      against an exception, it is simply true of every change in the repo — but
      relocate the branch mandate so it governs both paths instead of sitting
      inside "For non-trivial work". The failure to avoid is a reader taking the
      trivial path, never reaching step 3, and finding the branch requirement
      only in the paragraph that does not apply to them. If relocating turns out
      to require renumbering §1's steps, stop: that is a C2 escalation per
      *Conventions*.
      Then give the trivial path a branch name source per Open question 3 —
      non-trivial work takes its name from the plan file (§1 step 2), which
      trivial changes do not have.
- [x] 6. **Verify and align `docs/git-workflow.md:5`, `:280` and `:287`** (B4).
      Task 1 has already shifted these line numbers — the targets are, by
      heading: the first bullet of *Core Principles* ("Never commit directly to
      `main` or `develop`"), the first bullet of *After Merge* ("Delete the
      remote branch immediately"), and the first bullet of *Hard Rules* ("No
      force-push to `develop` or `main`").
      `:5` and `:287` should now read as consistent with `AGENTS.md` rather than
      contradicting it. Where it is worth saying that protection backs a rule up
      rather than the document merely asserting it, use the attribution form
      fixed in *Conventions* — no claim about when protection took effect.
      Reword `:280` per Open question 4.
      **Expect this task to change little.** `:5` and `:287` were always correct;
      it was `AGENTS.md` that contradicted them, and task 4 already fixed that
      side. If this task produces a large diff, something has been
      over-interpreted — re-read B4, which asks to *verify* consistency, not to
      manufacture it.
- [x] 7. **`CONTRIBUTING.md` pass** (B5). `:29–30` (plan-file/label workflow is
      the maintainer's own process) stays true and should not change. Check
      `:15` ("Target `develop`, not `main`") against task 3's finding and make
      it explicit that `--base develop` is required because the tool default is
      `main`. Also add what an external contributor now actually hits that is
      not written down: fork PRs require maintainer approval before any workflow
      runs (`all_external_contributors`, applied under #30), so CI staying idle
      on a fresh PR is expected and not a broken pipeline — the same class of
      "well-formed PR looks wrong" surprise the `Cargo.lock` note at `:23–26`
      already handles.
      **Verify that setting is live before documenting it.** It is the one claim
      in this plan taken from #30's comment rather than confirmed directly —
      `gh api repos/mrcmilano/mdv/actions/permissions/fork-pr-workflows` returns
      404 (wrong endpoint for a user-owned repo, not evidence of the setting's
      state), so confirm it in **Settings → Actions → General → Fork pull request
      workflows** or via a working API path. If it cannot be confirmed, leave the
      paragraph out and raise it in the C2 batch — telling contributors their CI
      needs an approval that does not exist is worse than silence.

**Part C — consistency review**

- [x] 8. **Read the three changed documents end to end** (C1) and produce a
      written findings list. Check specifically that:
      - no rule contradicts another rule in the same or another document;
      - no cross-reference points at a section that has moved, been renumbered,
        or no longer exists — `docs/git-workflow.md:19` cites "AGENTS.md §1 step
        4", `:226` and `:239` cite "§2", `docs/plans/PLAN-template.md:6,68`
        cites "§5", `AGENTS.md:61,67` cite "§3" and "§5". Under *Conventions*
        no §-numbering changes, so this should confirm rather than find; a hit
        here means a §-heading moved and the fix is to restore it, **not** to
        edit `PLAN-template.md`, which this plan does not permit changing;
      - every path an agent can take is covered — in particular the
        trivial-change path after Part B and the promotion path after Part A;
      - the documents agree on which branch is the base for feature work, for
        promotion, and for a fresh clone.
      Record the findings in the plan file under a `## C1 findings` section
      (added by this task — the template has none) and commit it, so they
      survive a context loss between this task and the next.
- [x] 9. **Formatter integrity check.** Before task 10's round-trip, confirm the
      editor-side formatter has not damaged the documents:
      - `grep -n '&gt;\|&lt;\|&amp;'` across the three files returns nothing that
        was not there on `develop`;
      - fenced blocks are balanced — the ` ``` ` count in each file is even, and
        every fence appearing in `git diff develop...HEAD` is one this plan
        intended to add (task 1's PR-body template, task 3's commands). **Do not
        compare the count to `develop`**: task 1 legitimately adds fences, so an
        unchanged count is the wrong test;
      - the full `git diff develop...HEAD` contains only intended edits — read
        it, do not skim the stat.
      Run this here and again before converting the PR to ready-for-review.
- [x] 10. **Batch every finding into one `needs-decision` round-trip** (C2) —
      the six Open questions above plus everything task 8 produced. One comment
      on #35 listing each item with its recommended resolution; then
      `gh issue edit 35 --remove-label agent --add-label needs-decision`.
      **Keep `in progress`** — it is still true, and the §5 table has no state
      for paused work; `needs-decision` alone carries the pause. The maintainer's
      side of this transition is §5's "Decision provided → remove
      `needs-decision`, add `agent`". Do not resolve ambiguities unilaterally and
      do not silently pick the reading that requires the least editing.
      **Stop here and wait for the maintainer** — commit and push everything
      first, per *Conventions*.
- [x] 11. **Apply the maintainer's resolutions** to the documents, then re-run
      task 9's integrity check. Only now are the documents final.
- [x] 12. **Added by `/skill:adversarial-review` (FIX REQUIRED, per `AGENTS.md`
      §4).** Three findings, all created or left standing by this branch:
      - **The A.2 guard could not detect the hazard it guards.** The promotion
        post-merge check was `git branch -r | grep origin/develop`, which reads
        remote-tracking refs; `fetch.prune` is unset in this repo, so a plain
        `git fetch` leaves a stale `origin/develop` behind and the grep reports
        "still exists" in precisely the deletion case. Replaced with
        `git ls-remote --heads origin develop`, which queries the remote.
        Verified: returns one line today.
      - **"CI is green" was unsatisfiable on a no-draft PR.** Task 11's C1-3
        lead-in said the checklist runs "immediately before opening" for trivial
        and promotion PRs, but workflows trigger only on `pull_request` and on
        pushes to `main`/`develop` — never on a feature-branch push (verified:
        every run on this branch is a `pull_request` event). Reordered so the CI
        item is checked on the open PR and the rest before opening.
      - **The conversion-presumption bug survived at a third site.** *Keeping
        Your Branch Up to Date* and *Responding to Review Feedback* both keyed
        the history-rewrite guard to the PR being *converted* to
        ready-for-review — an event that never occurs on a trivial or promotion
        PR, leaving the guard permanently un-triggered on a literal reading.
        Both now key off the PR *being* ready-for-review, converted or opened
        that way.
      Two LOW findings fixed in the same pass rather than deferred, both
      one-liners: the `Daily Workflow` `gh pr create` example omitted
      `--body-file` (opens an editor, blocks an agent, drops the §2 header
      block), and the promotion restore command pushed from local `develop`,
      which recreates an older tip from a stale clone — it now restores from
      parent 2 of the promotion merge. Both documented commands were verified
      against the real `#32` merge: `git rev-parse 549da73^2` → `07de96d`, and
      `^2` fails closed on a non-merge tip.

- [x] 13. **Added by `/skill:pre-merge-code-review` (F1–F5, maintainer approved
      2026-08-11).** Five findings, all created by this branch:
      - **F1 — `AGENTS.md` forbade the promotion PR this branch introduces.** The
        audience note (`:12–15`) and §1 (`:55–56`) asserted, without exception,
        that work lands through a PR *targeting `develop`* from a branch *created
        from `develop`* — which the promotion PR (head `develop`, base `main`)
        does not satisfy, and `AGENTS.md` never mentioned promotion at all. Task
        8's C1 checklist required the documents to "agree on which branch is the
        base … for promotion"; they did not. Fixed at all three sites: the
        audience note drops the false absolute (outside PRs still target
        `develop`, per `CONTRIBUTING.md`), §1 gains a paragraph naming the
        `main`-targeting PRs, and §2's Scope names them as no-draft paths.
      - **F2 — `docs/git-workflow.md` contradicted itself.** `:59` said "the one
        PR that legitimately targets `main` is a promotion"; `:195`, added by
        task 2 in this same branch, opens a revert PR with `--base main`. Both
        `:59` and F1's new `AGENTS.md` paragraph now name **two**
        `main`-targeting PRs — promotion and revert-of-a-bad-merge.
      - **F3 — the revert recipe omitted the body argument this branch made
        mandatory.** `:81–84` added "Always pass `--body-file`: without a body
        argument `gh` opens an editor, which blocks an autonomous agent", while
        the revert `gh pr create` passed only `--title`. Now passes
        `--body-file`.
      - **F4 — *Before Marking Ready* contradicted itself within three
        sentences:** "**Not before opening:**" followed by "and the rest before
        you open it". The bold clause was meant to scope only the CI item;
        reworded to say so.
      - **F5 — the trivial keeps-list pointed into non-trivial-only text.** It
        sent the reader to "step 3 below" for the branch requirement, but step 3
        now reads "using the name given in the plan file" — which a trivial
        change does not have. It now points at §1's universal branch rule, where
        trivial branch naming actually lives.
      **Consequential edits forced by F2**, not independent findings: *Opening*
      now lists **three** kinds of PR that skip the draft stage (the revert PR
      opens ready-for-review and always did); the *Before Marking Ready* rebase
      item exempts both `main`-targeting PRs, since a revert must stay based on
      `main`; and the *Solo-workflow rule*'s inline example list was replaced
      with a pointer to *Opening*, so the enumeration lives in exactly one place
      and cannot drift again.
      **F6 (fixed in the same pass, maintainer requested).** The revert recipe's
      step 4 verified "main is clean" with `git diff origin/develop origin/main`
      and stated no expected result — a check that is legitimately non-empty
      whenever `main` lags `develop`, which is normal operation, and which the
      promotion section's "expect empty" framing invites a reader to
      misinterpret. Replaced with `git diff <merge-commit-sha>^1 origin/main`,
      which compares `main` against the state it was in before the bad merge
      landed and is exactly empty when the revert succeeded, plus the
      not-empty case. A note under the block rules the `develop`↔`main` diff out
      explicitly and names it as the *promotion* success condition instead.
      Verified against the real `#32` merge: `git rev-parse 549da73^1` →
      `f29b108`.
      **Deliberately not changed:** F7 (the documents state branch protection as
      live; `rulesets` is `[]` and both branches return 404 today). This is the
      maintainer-approved resolution to Open question 6 — the wording is applied
      when #30 lands, not before.

- [x] 14. **Added by a second `/skill:adversarial-review` pass** over task 13's
      commits (`AGENTS.md` §4). Six findings, all created or sharpened by the
      task-13 edits; two were FIX REQUIRED.
      - **FIX REQUIRED — the rebase instructions collided with *Hard Rules*.**
        *Keeping Your Branch Up to Date* and *Resolving Conflicts Before Merge*
        both said "rebase onto `develop`" and force-push, unconditionally, of
        "a PR". A promotion PR's head branch **is** `develop`, so following the
        conflict recipe on one is a force-push to `develop` — forbidden "under
        any circumstances" by *Hard Rules*. On a revert PR the same instruction
        drags all of `develop` into a `main`-based branch. Task 13 had exempted
        these two PRs in *Before Marking Ready* only; both other sites now carry
        a **Feature branches only** guard, placed *above* the command block in
        the conflict section so it is read before the command is copied. A
        promotion PR that conflicts is an escalation (its precondition failed),
        and a revert PR rebases onto `origin/main`.
      - **FIX REQUIRED — the two `main`-targeting PRs had no defined process.**
        §1 declares its trivial/non-trivial lists exhaustive and warns against
        carrying steps over by inference, but task 13 added a third category to
        neither list — leaving "does a promotion need a plan file?" unanswerable,
        which is Open question 5's failure mode on a new path. **Maintainer ruled
        2026-08-11:** both are procedures, not features — no plan file, no §5
        label transition, no §3 test loop, no step 5 Finish;
        `docs/git-workflow.md` is their complete specification. Stated in §1,
        along with the qualifier that the two lists stay exhaustive *for ordinary
        work*. **Accepted risk, recorded:** a revert onto `main` therefore gets
        no adversarial review.
      - **Unsatisfiable check (medium).** Step 4's
        `git diff <merge-commit-sha>^1 origin/main` can never be empty if
        anything legitimately landed on `main` after the bad merge — the same
        defect class as task 12's "CI is green" finding. The recipe now states
        the assumption and gives the fallback check for that case.
      - **Promotion with nothing to promote (low).** An empty payload made
        `gh pr create` fail with an opaque "No commits between main and develop".
        Precondition 1 now names the empty case and says to stop.
      - **`hotfix:` had no path (low).** Task 13's absolute "Nothing else targets
        `main`" slammed a door the Conventional Commits type list at `:10` opens.
        *Branch Strategy* now says there is no separate hotfix path: an urgent
        fix branches from `develop` and reaches `main` by promotion.
      - **Dangling reference (low).** §2's "open ready-for-review **for the same
        reason**" pointed at a reason stated in neither §1 nor §2. Replaced with
        the reason itself.
      **Not fixed, accepted:** step 4's `<merge-commit-sha>^1` returns a
      plausible-but-wrong answer if a non-merge SHA is substituted — reachable
      only by skipping step 2, which fails loudly first
      (`git revert -m 1` rejects a non-merge). Verified in-environment that the
      caret syntax survives the shell (`extendedglob` unset;
      `git rev-parse 549da73^1` → `f29b108`).

- [x] 15. **Added by a second `/skill:pre-merge-code-review` pass (F1–F9,
      maintainer approved 2026-08-11).** The review built the cross-product of
      every PR *shape* the documents now describe against every rule keyed to a
      base branch, head branch, rebase target, draft state or force-push —
      including rules this branch never touched, which is where most of these
      were. Six shapes exist, not the two the prose counts: non-trivial feature
      PR, trivial PR, promotion PR, revert-of-bad-merge PR, external/fork PR,
      and the re-landed feature PR after a revert.
      - **F1 (high) — a revert onto `main` permanently broke every later
        promotion, twice over.** Verified by executing the documented recipes in
        a throwaway repo, not by reading them. (a) `git revert -m 1` leaves a
        **non-merge** commit on `main`, so promotion precondition 2
        (`--no-merges origin/develop..origin/main`, "expect empty") is non-empty
        from then on *forever* — and the surrounding prose reads that as
        divergence and says stop and escalate, halting every future promotion on
        a false alarm. (b) Worse, step 5's "the feature branch is untouched —
        open a new PR targeting `develop`" loses the feature: its commits are
        already in `main`'s history through the bad merge, so the next promotion
        has nothing to apply and `main` stays permanently without the work,
        while the promotion's own "expect empty" diff check fails with no stated
        cause. Precondition 2 now names the revert commit as the one legitimate
        non-merge commit on `main`; step 5 now re-lands the work with **fresh**
        commits (cherry-pick onto `develop`) and explains why. Both remedies
        were tested; the cherry-pick was chosen over revert-the-revert because
        the latter would add a **third** `main`-targeting PR shape, which is
        exactly the enumeration tasks 13–14 fixed.
      - **F2 (high) — the `develop` restore command silently restored the wrong
        commit.** The comment claimed "`^2` errors out rather than pushing the
        wrong commit". False: `git rev-parse` prints its own argument back on
        stdout when it fails, so the substitution is never empty, and that
        fail-closed behaviour holds only when `main`'s tip is a **non-merge**
        commit — a state this workflow forbids, since `main`'s tip is always a
        promotion or revert-PR merge. Demonstrated in a sandbox: with `develop`
        deleted and a non-promotion merge at `main`'s tip, the command recreated
        `develop` at a `main`-based branch tip, exit 0, no warning. Now names the
        promotion merge explicitly and gates the push on `rev-parse --verify`.
      - **F3 (medium-high) — no-draft PRs could never rebase, yet the conflict
        recipe ordered them to.** Task 12/14 keyed the history-rewrite guard to
        the PR *being* ready-for-review ("converted or opened that way"), which
        for the three no-draft kinds is true at t=0, while *Resolving Conflicts
        Before Merge* keyed the same permission to "no inline review comments"
        and instructs rebase + `--force-with-lease`. A conflicted trivial PR was
        simultaneously required and forbidden to take the only remedy offered.
        All sites now key off the single thing the guard protects: an inline
        review comment existing.
      - **F4 (medium) — "Fix on the same branch" instructed a direct push to
        `develop`.** Unqualified at *Responding to Review Feedback* and *CI
        Failures*; a promotion PR's head branch **is** `develop`. Task 14 had
        guarded the rebase and force-push sites but not the plain-push ones.
        Both now carry the guard.
      - **F5 (medium) — *Branched from Wrong Base* mis-fired on the revert
        shape.** Untouched by this branch, and its `rebase --onto develop main`
        is precisely wrong for a branch whose `main` base is deliberate. Now
        excluded explicitly.
      - **F6 (medium) — revert step 5 assumed a branch the repo had already
        deleted.** `deleteBranchOnMerge` is `true`, so the bad merge deleted the
        feature branch on the remote — the same mechanism the promotion section
        devotes a whole check to. Folded into the step 5 rewrite.
      - **F7 (medium) — "Do not reuse the branch name" was unsatisfiable for
        both `main`-targeting shapes.** A promotion's head is always `develop`;
        the revert recipe hard-codes its branch name. Scoped to feature branches.
      - **F8 (low-medium) — `docs/git-workflow.md` had no audience statement**,
        so every rule bound external contributors, including the draft-PR
        mandate that the "three kinds" enumeration does not cover, and a
        "CI is green" item a fork contributor cannot satisfy alone. Added an
        audience note, a fork bullet under *Opening*, a qualifier on the CI item,
        and a pointer from `CONTRIBUTING.md`.
      - **F9 (low) — the documents disagreed on whether protection is live.**
        *Core Principles* and `AGENTS.md` §1 asserted it as enforced now; the
        promotion section's restore procedure exists only because it is not.
        **Maintainer ruled 2026-08-11:** protection is not live and goes live
        when #30 lands. Both sites now say so, which also matches the "until
        then" framing the promotion section already used. This supersedes the
        Open question 6 / task 13 F7 resolution, which had left the assertion
        standing.

### Finish
- [x] Write / update tests for all implementation tasks above
      — **not applicable: documentation only, no code changes.** State this
      explicitly in the PR rather than leaving the box ambiguous.
- [x] Run full test suite — all tests pass (baseline confirmation that nothing
      in the branch touched code)
- [x] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
      The skill's code-oriented checks (panics, overflow, rendering, terminal
      resize) are vacuous on a docs-only branch — do not skip the step on that
      basis. Point it at the documents: a rule that cannot be followed, a
      procedure with a missing step, an instruction that contradicts another.
- [x] Update `README.md` if affected
      (verified 2026-08-11: `README.md`'s only workflow-related line is the
      `CONTRIBUTING.md` link at `:85`, so this is expected to be a no-op —
      confirm rather than assume)
- [x] Convert draft PR to ready-for-review; add `Closes #35` to PR description;
      set this plan's `_Status:_` to `READY`
- [x] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 35 --remove-label agent --remove-label "in progress" --add-label needs-review`
- [ ] **After merge, note in #30 that its `#35` prerequisite is satisfied** —
      #30 is the only consumer of this issue and its checklist is gated on it.
      The promotion PR comes first; task 1's new section is what runs it.

---

## C1 findings

Produced by task 8, after tasks 1–7 landed. Read end to end: `AGENTS.md`,
`docs/git-workflow.md`, `CONTRIBUTING.md`. Every finding is carried into the
task 10 batch with the recommended resolution; none is applied unilaterally.

**Cross-references verified clean.** No `§` heading, `§1` step number or section
title changed. `docs/git-workflow.md:22` ("AGENTS.md §1 step 4"), `:261` and
`:274` ("§2"), `docs/plans/PLAN-template.md:6,68` ("§5"), and `AGENTS.md`'s own
`§2/§3/§5` citations all still resolve. `docs/git-conflict-resolution.md` and
`README.md` contain no rule touched by this branch. This confirms rather than
finds, as the plan predicted.

**C1-1 — `docs/git-workflow.md` *Pull Requests > Opening* still mandates a draft
PR for every PR.** "Open a draft PR immediately after the first push" is
unconditional, and now contradicts `AGENTS.md` §1, where a trivial change opens
ready-for-review, and also the promotion PR, which has no WIP stage. → Recommend
qualifying the bullet: it governs non-trivial feature work, with a pointer to
the two exceptions.
**RESOLVED 2026-08-11 — maintainer approved; applied in task 11.** The bullet
now reads "on non-trivial feature work", followed by a bullet naming both
exceptions (trivial changes, promotion PRs) as opening ready-for-review.

**C1-2 — `AGENTS.md` §2 states the same rule unconditionally from the other
side.** §2 opens "After the first push on a new branch, open a draft PR
targeting `develop` immediately". §1's skip-list says the trivial path skips it,
but a reader who lands in §2 directly — which §1 step 4 and
`docs/git-workflow.md` both invite — never sees that. This is the exact failure
mode Open question 5 warns about, reintroduced one section later. → Recommend a
scope line at the top of §2 pointing back to §1's trivial path.
**RESOLVED 2026-08-11 — maintainer approved; applied in task 11.** §2 now opens
with a **Scope:** paragraph scoping the draft stage to non-trivial work and
stating what the trivial path does instead, including where §2 stops being
path-specific (*Before any subsequent git operation* onward applies to both).

**Consequential edits made under the same resolution**, because leaving them
would have reintroduced the ambiguity the resolution exists to remove. Each is
forced by the decision, not an independent finding:

- `AGENTS.md` §2's `Closes #<N>` rule read "not until converting to
  ready-for-review", which presumes a conversion a trivial PR never makes.
  Reworded to key off *being a draft*, with the no-draft-stage case stated.
- `docs/git-workflow.md` *Description* → `## Why` carried the same presumption
  ("until the PR is converted … then add `Closes #<N>`"). Same rewording.
- `docs/git-workflow.md` *Daily Workflow* step 4 was the third site asserting
  "open a draft PR" unconditionally. Now "open the PR … (draft on non-trivial
  work — see Pull Requests > Opening)". This touches the block C1-8 proposes to
  extend with a `gh pr create` line; the two edits do not conflict.

**Post-resolution sweep.** Every remaining mention of a draft PR,
ready-for-review or `Closes` across the three documents is now either
conditional on the PR actually being a draft, or explicitly scoped to one path:
`AGENTS.md:79,89,155,174–178,180–181,190–193,239` and
`docs/git-workflow.md:80,97,100,254,257,267,277–281,297`. No site states the
draft rule unconditionally.

**C1-3 — *Before Marking Ready* has no moment to run on a trivial PR, and one
item is wrong for a promotion PR.** A trivial PR opens ready-for-review, so a
checklist keyed to the draft→ready transition never fires; and "Branch is
rebased on latest `develop`" is meaningless for a promotion PR whose head *is*
`develop`. → Recommend: state that the checklist runs before opening when there
is no draft stage, and exempt promotion PRs from the rebase item.

**C1-4 — Which PR body does a promotion PR use?** *Pull Requests > Description*
says "Every PR body must include the following sections" (What / Why / Notes for
reviewer); the new promotion section supplies its own body template that
deliberately does not. Both are mandatory as written. → Recommend the promotion
template win for promotion PRs, said explicitly in one of the two places.

**C1-5 — An issue-driven trivial change leaves its issue with no terminal
state.** The skip-list drops the §5 label workflow wholesale, so a trivial fix
raised from an issue never clears `agent` and never reaches `needs-review`; the
issue stays open in the state machine forever. The §5 table has no trivial-path
row. → Recommend the trivial path skip §5 only for non-issue-driven work, and
otherwise still run the closing transition (`--remove-label agent --add-label
needs-review`) when the PR opens.

**C1-6 — the audience note now scopes the branch-and-PR mandate as
maintainer-only.** `AGENTS.md:8–12` says `## Workflow (required)` (§1–§5) is the
maintainer's own process and "not expected of outside contributions" — but §1
is now also where the universal "every change is made on a branch and lands
through a PR" rule lives. An external reader can correctly conclude the PR
requirement does not apply to them. `CONTRIBUTING.md` covers them in practice,
so this is a reading hazard rather than a live failure. → Recommend the audience
note distinguish the universal branch/PR rule from the plan-file and label
ritual that genuinely is maintainer-only.

**C1-7 — pre-existing formatter damage at `docs/git-workflow.md:11–14`.** The
four example commit messages are single-line ` ```…``` ` spans rather than one
fenced block — the same collapse pattern the Risks section attributes to the
editor-side formatter. Present on `develop`, so not caused by this branch, and
formatter-hunting is out of scope; the damage itself sits in a document this
task is auditing. → Recommend repairing it into a single fenced block as part of
task 11.

**C1-8 (minor) — *Daily Workflow* step 4 never shows the command its own warning
requires.** The step is commented "Push and open a draft PR targeting develop"
but contains only `git push -u origin feature/my-change`; the `gh pr create
--base develop` that *Branch Strategy* now insists on is absent from the one
place a reader copies commands from. → Recommend adding the command to the
block.

---

## C2 resolutions

**Maintainer ruled 2026-08-11: all C1 recommendations approved as written.**
C1-1 and C1-2 were applied first (annotated inline above); C1-3 through C1-8
followed in task 11. What landed:

| Finding | Applied |
|---|---|
| C1-3 | *Before Marking Ready* gains a lead-in — the checklist runs at draft→ready, or immediately before opening when there is no draft stage — and the rebase item is marked not applicable to a promotion PR. |
| C1-4 | *Description* now exempts a promotion PR, which uses the body from *Promoting `develop` → `main`*. |
| C1-5 | The §5 skip is now conditional: an issue-driven trivial change still runs the closing transition when its PR opens; a non-issue-driven one touches no labels. §5 gains a matching trivial-path transition block, so the rule is stated on both sides. |
| C1-6 | The audience note now names the branch-and-PR rule as the exception to the maintainer-only scoping — what an outside contribution skips is the plan file, labels and review ritual, never the PR. |
| C1-7 | `### Example Commit Messages` repaired from four single-line ` ``` ` spans into one fenced block. |
| C1-8 | *Daily Workflow* step 4 now carries `gh pr create --draft --base develop`, with the `--base` requirement and the trivial-path `--draft` exception in comments. |

**Fence-count movement is expected here and is not formatter damage:**
`AGENTS.md` 14 → 16 (C1-5's new `bash` block in §5) and `docs/git-workflow.md`
42 → 40 (C1-7 replaces four single-line pseudo-fences with one two-line block).
Both are accounted for; entity scan stays clean.

**C1-5 interacts with the C1-2 resolution.** Because a trivial PR now carries
`Closes #<N>` from the start, the issue auto-closes on merge — so the label
transition is what keeps the *state machine* honest, not what closes the issue.
Both are written down rather than left to interact by accident.

## Open question A.2 — auto-delete of `develop`, confirmed

Re-verified 2026-08-11 against the live repo, since the maintainer asked whether
`develop` can be lost:

- `develop` is **unprotected** (`branches/develop/protection` → 404) and
  `rulesets` returns `[]` — there is no ruleset at all yet. It is also not the
  default branch (`main` is). GitHub's auto-delete skips protected and default
  branches, so `develop` currently matches neither exemption.
- **Therefore the exposure is real, not theoretical:** plan for `develop` to be
  deleted when a promotion PR merges.
- **No commits can be lost, and this is provable from the last promotion.** The
  `#32` merge commit `549da73` has parents `f29b108` (main's tip) and `07de96d`
  (develop's tip at merge time). A promotion merge always records develop's exact
  tip as parent 2, so after the merge every commit on `develop` is reachable from
  `main`. Deletion removes a *ref*, never history.
- Recovery is exact and one command — `git push origin develop` from an
  up-to-date local clone, or GitHub's **Restore branch** button on the merged PR.
- `#32` merged 2026-08-07, before `deleteBranchOnMerge` was enabled on
  2026-08-11, so it provides no evidence either way — the first promotion after
  this plan lands is the first exposed one.

**Accepted as a side effect**, on the reasoning that the loss is ref-only,
recovery is exact, and the window is closed permanently by #30. The promotion
section's post-merge check exists precisely to catch it in that window.
**Recommended sequencing, which removes the hazard entirely:** apply #30's
protection to `develop` *before* running the first promotion. Protecting
`develop` restricts pushes *to* `develop`; a promotion PR has `develop` as head
and `main` as base, so it is unaffected — while the protection makes `develop`
exempt from auto-delete.
