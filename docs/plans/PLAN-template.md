# PLAN: <feature name>

_Branch:_ `<type/feature-name>`
_Date:_ YYYY-MM-DD
_Status:_ DRAFT
<!-- DRAFT        → APPROVED:      user approves via label flip (needs-review → agent, see AGENTS.md §5) -->
<!-- APPROVED     → IN PROGRESS:   first code task commit (task 1+); -->
<!--                              branch creation (task 0) does NOT flip it -->
<!-- IN PROGRESS  → READY:         PR converted to ready-for-review -->
<!-- READY is the terminal status this file records. Actual completion -->
<!-- ("done") only occurs when the PR is merged, by which point the plan -->
<!-- file is no longer being updated. -->
<!-- Each transition: update this field and commit the plan file. -->
_Source:_ #<issue-number>
<!-- omit _Source_ line entirely if not GitHub-issue-driven      -->
_PR:_ #<pr-number>
<!-- filled in after first push; omit until then                 -->

---

## Problem
<!-- One paragraph. What is being built and why. -->

## Out of scope
<!-- Be explicit. Anything not listed here is fair game for scope creep. -->

## Impact assessment
<!-- Which files, modules, or layers are affected. Any risks or dependencies. -->

## Open questions
<!-- Format: question → [BLOCKS | ASSUMPTION: <what agent will assume if not answered>] -->
<!-- A BLOCKS question must be resolved before implementation starts.       -->
<!-- An ASSUMPTION question: agent proceeds with the stated assumption.     -->
<!-- Remove this section once all questions are resolved.                   -->

---

## Tasks

### Implementation
- [ ] 0. Create branch `<branch-name>` from develop following docs/git-workflow.md
- [ ] 1. <!-- atomic step — one behaviour or one file -->
- [ ] 2.
<!-- Keep tasks small enough that each one can be completed and tested      -->
<!-- independently. If a task feels too large, split it.                    -->
<!-- After each completed task: commit using a Conventional Commit subject     -->
<!-- (per docs/git-workflow.md), task + issue ref appended inline:            -->
<!--   issue-driven:     `<type>: <description> (task <N>, ref #<issue>)`     -->
<!--                     e.g. `feat: add TOC overlay (task 3, ref #42)`       -->
<!--   not issue-driven: `<type>: <description>`                              -->
<!--                     e.g. `feat: add TOC overlay`                        -->
<!-- Initial push: push the branch immediately after the plan-file commit     -->
<!-- (task 0), which triggers the draft PR. Thereafter push after every 2     -->
<!-- completed tasks, before any risky git operation (rebase, merge, file     -->
<!-- deletion), or before ending a session — whichever comes first.           -->
<!-- "Ending a session" = before yielding control back to the user.           -->

### Finish
- [ ] Write / update tests for all implementation tasks above
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected
- [ ] Convert draft PR to ready-for-review; add `Closes #<N>` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit <N> --remove-label agent --remove-label "in progress" --add-label needs-review`
      (omit if not GitHub-issue-driven)

---

<!-- Add this section only if adversarial-review produced deferred LOW findings -->
## Deferred findings
<!-- Format: [LOW] <finding> — <rationale for deferral> — <follow-up issue if any> -->
