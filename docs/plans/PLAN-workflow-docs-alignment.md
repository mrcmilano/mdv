# PLAN: Align workflow documentation with branch protection

_Branch:_ `chore/issue-35-workflow-docs`
_Date:_ 2026-08-11
_Status:_ IN PROGRESS
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
- [ ] 9. **Formatter integrity check.** Before task 10's round-trip, confirm the
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
- [ ] 10. **Batch every finding into one `needs-decision` round-trip** (C2) —
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
- [ ] 11. **Apply the maintainer's resolutions** to the documents, then re-run
      task 9's integrity check. Only now are the documents final.

### Finish
- [ ] Write / update tests for all implementation tasks above
      — **not applicable: documentation only, no code changes.** State this
      explicitly in the PR rather than leaving the box ambiguous.
- [ ] Run full test suite — all tests pass (baseline confirmation that nothing
      in the branch touched code)
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
      The skill's code-oriented checks (panics, overflow, rendering, terminal
      resize) are vacuous on a docs-only branch — do not skip the step on that
      basis. Point it at the documents: a rule that cannot be followed, a
      procedure with a missing step, an instruction that contradicts another.
- [ ] Update `README.md` if affected
      (verified 2026-08-11: `README.md`'s only workflow-related line is the
      `CONTRIBUTING.md` link at `:85`, so this is expected to be a no-op —
      confirm rather than assume)
- [ ] Convert draft PR to ready-for-review; add `Closes #35` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
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

**C1-2 — `AGENTS.md` §2 states the same rule unconditionally from the other
side.** §2 opens "After the first push on a new branch, open a draft PR
targeting `develop` immediately". §1's skip-list says the trivial path skips it,
but a reader who lands in §2 directly — which §1 step 4 and
`docs/git-workflow.md` both invite — never sees that. This is the exact failure
mode Open question 5 warns about, reintroduced one section later. → Recommend a
scope line at the top of §2 pointing back to §1's trivial path.

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
