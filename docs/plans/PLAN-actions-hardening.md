# PLAN: GitHub Actions hardening

_Branch:_ `chore/issue-33-actions-hardening`
_Date:_ 2026-08-11
_Status:_ IN PROGRESS
_Source:_ #33

---

## Problem

The two workflows (`.github/workflows/ci.yml`, `.github/workflows/audit.yml`)
were written while the repo was private, and carry three problems that only
matter now that it is public and about to be branch-protected:

1. **A latent fork-PR break.** `rustsec/audit-check@v2.0.0` publishes its result
   through the Checks API, which needs `checks: write`. On a `pull_request`
   event from a fork, GitHub caps `GITHUB_TOKEN` at read-only regardless of
   repository settings or any `permissions:` block, so the action 403s and the
   job goes red. The job is gated on `Cargo.lock` having changed, so it stays
   dormant until the first outside PR that adds, removes or bumps a dependency
   — at which point that contributor gets a red check they cannot fix.
2. **No aggregate check.** #30's branch-protection ruleset would need to name
   eight matrix-expanded check runs. Renaming any matrix leg then makes a
   required check never report, and PRs block forever with no error message.
3. **Implicit token grants and unpinned actions.** The effective posture is
   correct today, but it rests on repository settings rather than on anything
   the workflows assert.

This plan implements items 1, 3, 4, 5 and 6 of #33. **Item 2 (weekly `schedule:`
cron) was cut during plan review** — see Out of scope. One small addition was
confirmed during review: `workflow_dispatch:` on the audit workflow (task 2).

## Out of scope

- **Item 2 of #33 — the weekly `schedule:` cron. Cut during plan review.**
  #33 justifies it as closing a total gap: *"a new RUSTSEC advisory against
  `pulldown-cmark` tomorrow would never be caught by CI, because nothing in the
  repo changed."* That premise is false — **Dependabot vulnerability alerts are
  already enabled on this repo** (`GET /repos/mrcmilano/mdv/vulnerability-alerts`
  → `204`, with `automated-security-fixes` enabled). The GitHub Advisory
  Database ingests RUSTSEC advisories for Cargo, so GitHub already watches a
  static `Cargo.lock` and alerts within hours, versus up to seven days for a
  weekly cron.
  A cron's real marginal coverage was therefore only the `unmaintained` /
  `unsound` / `yanked` classes that Dependabot does not surface — findings
  which are planning signals, not exploitable conditions, and which the
  `Cargo.lock`-change path catches anyway. Against that: GitHub disables
  scheduled workflows after 60 days of repository inactivity, so the cron would
  switch itself off exactly when it was the only thing auditing; and under
  `--deny warnings` an unfixable unmaintained advisory would turn it red every
  week until ignored, which is how a maintainer learns to ignore a cron.
  **Do not reintroduce a `schedule:` trigger as part of this issue.** If it is
  ever revisited, revisit it against the corrected premise above, not #33's.
- **Branch protection / required status checks** (#30). This plan creates the
  `ci-success` check that #30 will require; it does not configure protection.
- **Actions `allowed_actions` allowlist** (#36). Depends on this issue settling
  the final action list, but is separate work.
- **Workflow documentation** (#35).
- **MSRV** (#29). #29 will move the `dtolnay/rust-toolchain@stable` refs to
  `@1.xx.0`. #33's item 5b (task 5 here) is the reason not to SHA-pin those
  lines now.
- **Upgrading `actions/checkout` v4 → v7.** Decided during Assess: pin the
  current `v4` tag's SHA and stop there. A major-version bump has a different
  risk profile from a hardening PR (v4 runs on the Node 20 runtime, v5+ on
  Node 24) and is not what #33 asks for. See "Follow-up" below.
- **Build caching** (`Swatinem/rust-cache` or similar). Same reasoning as
  `PLAN-ci-workflow.md`: keep the CI trust surface minimal.
- **Any change to `Cargo.toml`, `Cargo.lock`, or Rust source.** This is
  CI configuration only. (Task 7 pushes a *transient* Rust-source change to
  prove the `ci-success` red path; it is force-dropped before the PR is
  converted and does not survive into the merged branch.)
- **Rewording `AGENTS.md`.** Decided during review: CI uses
  `cargo audit --deny warnings` while `AGENTS.md:34`'s local convenience
  command stays bare. See "Accepted consequences" below.

## Impact assessment

Two files change. No Rust source is touched by any *durable* change — task 7.4
pushes a deliberately malformatted source file to prove the `ci-success` red
path, then force-drops that commit off the branch.

### `.github/workflows/audit.yml`

| Change | Item |
|---|---|
| `permissions: contents: read` at workflow level | 4b |
| Comment above `on:` guarding against `pull_request_target` | 6a |
| `rustsec/audit-check@v2.0.0` → `dtolnay/rust-toolchain@stable` + `cargo install cargo-audit --locked` + `cargo audit --deny warnings` | 1a |
| `workflow_dispatch:` trigger + explicit gate branch | — (added during review) |
| `actions/checkout` SHA-pinned; comment on the toolchain branch ref | 5a, 5b |

### `.github/workflows/ci.yml`

| Change | Item |
|---|---|
| `permissions: contents: read` at workflow level | 4a |
| Comment above `on:` guarding against `pull_request_target` | 6a |
| New `ci-success` aggregate job | 3a |
| `actions/checkout` SHA-pinned (3 uses); comment on the toolchain branch ref | 5a, 5b |

### Decisions taken during Assess (confirmed by the maintainer)

- **`actions/checkout` pin target.** The `v4` tag currently resolves to
  `11d5960a326750d5838078e36cf38b85af677262`, which is **v4.4.0** — not the
  older v4.2.2 that most examples cite. Pin that SHA with a `# v4.4.0`
  trailing comment (more precise than a bare `# v4`, and it records exactly
  what was pinned).
- **Cron scope.** Moot — the cron was subsequently cut. Retained only to
  explain why no `schedule:` appears anywhere in this plan.

### Decisions taken during plan review (all confirmed by the maintainer)

- **Audit strictness: `cargo audit --deny warnings`.** Verified against
  `cargo audit --help`: `-D, --deny` takes `warnings (any), unmaintained,
  unsound, yanked`, meaning **bare `cargo audit` exits 0 on unmaintained,
  unsound and yanked advisories** and fails only on vulnerabilities.
  `AGENTS.md:310` states the policy as "fail on any advisory", so a bare
  invocation would have installed a CI gate weaker than the project's own
  documented rule — silently, since the job would be green either way.
  Verified clean today: `cargo audit --deny warnings` exits 0 across all 42
  crates, so this does not arrive red.
- **`workflow_dispatch:` (task 2) is confirmed IN**, not optional — but on
  narrower grounds than first written. It was originally justified as the only
  way to exercise the new `cargo audit` path on this PR; that is false, because
  GitHub keeps a `workflow_dispatch` trigger inert until it exists on the
  default branch, and this PR targets `develop`. Kept on its remaining merit
  alone: an on-demand audit after a manual `cargo update`, the step `AGENTS.md`
  prescribes but cannot enforce. Verification of the audit path moved to a
  throwaway commit (task 7.3).
- **Weekly cron: cut.** See Out of scope for the full reasoning.

### Accepted consequences

- **CI is stricter than the documented local command.** `AGENTS.md:34` keeps
  bare `cargo audit`; CI runs `--deny warnings`. This is deliberate — the
  local command is a convenience check, CI is the gate, and `AGENTS.md:310`
  ("fail on any advisory") describes the gate accurately once this lands. The
  practical effect is that a local `cargo audit` can pass while CI fails on an
  unmaintained-crate advisory. **Do not "fix" this by changing `AGENTS.md`** —
  rewording it was offered and declined.
- **An unfixable advisory can block PRs once #30 lands.** With `--deny
  warnings` on a required check, an unmaintained-crate advisory with no
  upstream fix turns `audit` red with no code change able to clear it. The
  escape hatch is `cargo audit --ignore <RUSTSEC-ID>` with a comment naming
  the advisory and why it is tolerated. Do **not** add any `--ignore` flag
  pre-emptively in this PR — there is nothing to ignore today.
  Note this risk is now confined to PRs that touch `Cargo.lock`, since cutting
  the cron removed the only path that could surface a warning without a
  lockfile change.
- **Vulnerability monitoring against a static lockfile is Dependabot's job,
  not CI's.** That is a deliberate division of labour after cutting the cron,
  not an unmonitored gap. If Dependabot alerts are ever disabled on this repo,
  that gap reopens and the cron decision should be revisited.

### Explicit non-changes (do not "improve" these)

- **Keep `fetch-depth: 0` on the audit checkout.** The `workflow_dispatch`
  branch short-circuits before any `git diff`, so full history looks wasteful
  on that path — but the `push` and `pull_request` paths need it for
  `git diff "$base_sha"`. Removing or shallowing it breaks the lockfile gate.
- **Leave `cargo install cargo-audit --locked` unpinned as to version.**
  `--locked` makes each install reproducible for whatever version resolves;
  the version itself floats deliberately, because advisory tooling is exactly
  the thing that should not be frozen. Do not add `--version`.
- **Keep the three CI jobs (`fmt`, `clippy`, `test`) exactly as they are.**
  `ci-success` is additive. Do not merge, rename or restructure them —
  renaming a matrix leg is the specific failure #30 is trying to avoid.
- **Do not rename the `audit` job either.** #30 will require it as a status
  check by that exact name, and unlike the `ci.yml` jobs it has no aggregate
  in front of it — a rename breaks #30 directly. The same applies to the
  workflow-level `name:` keys (`CI`, `Audit`), which the README badges resolve
  against.

### Things that changed since #33 was written

- **`audit.yml` has no Rust toolchain step today.** It relies on
  `rustsec/audit-check` bringing its own. Replacing the action with plain
  `cargo audit` means the job must install a toolchain explicitly; the
  runner's preinstalled Rust is unpinned and can drift. Task 1 adds
  `dtolnay/rust-toolchain@stable` to the audit job. This is a consequence of
  item 1, not new scope.
- **#33's cost argument for item 1 stands, now that the cron is cut.** #33
  reasoned that the `cargo install cargo-audit` compile "runs a handful of
  times a year" because the job is `Cargo.lock`-gated. Adding a weekly cron
  would have made that ~52 runs/year and invalidated the reasoning; cutting
  the cron restores it. The compile cost is a non-issue either way (Actions
  minutes are unmetered on public repositories), but the stated rationale is
  now accurate rather than merely harmless.
- **The dependency surface is 41 third-party crates, not 3.** #33 reasons from
  "three direct dependencies"; `Cargo.lock` holds 42 packages. This does not
  change any decision here, but it is the correct number to reason from in
  #29/#36 and in any future revisit of the cron question.

### Risks

- **The change is not verifiable by `cargo test`.** No Rust code changes, so
  the local test loop is trivially green and proves nothing. The real signal is
  a CI run, which requires the branch to be pushed and the draft PR opened.
  Task 7 covers this explicitly.
- **The audit steps are `Cargo.lock`-gated, and this PR does not touch
  `Cargo.lock`** — so on the PR itself they will be *skipped*, and a green
  `audit` check would prove nothing about whether `cargo audit` actually works.
  Task 7.3 handles this with a throwaway commit that forces the gate open.
  `workflow_dispatch` (task 2) is **not** available for this — GitHub keeps the
  trigger inert until it reaches the default branch.
- **A naive `needs:` aggregate reports success on a skipped or cancelled
  dependency**, which is the exact failure #33 warns about. Task 4 pins down
  the correct form and task 7 verifies the red path empirically rather than by
  reading the YAML.
- **README badges are unaffected.** Both workflows keep their `name:`
  (`CI`, `Audit`), and badge URLs reference the workflow file path, so
  `README.md:3-4` stay correct. No README change is expected.

---

## Tasks

### Implementation

- [x] 0. Create branch `chore/issue-33-actions-hardening` from develop following docs/git-workflow.md
      **Run this from the primary worktree `/Users/marco/Build/mdv`**, which is
      where `develop` is checked out. Three other worktrees exist on unrelated
      branches (`main-2`, `chore/issue-24-…`, `fix/issue-18-…`); starting from
      one of those would branch off the wrong base. Confirm with
      `git branch --show-current` → `develop` before creating the branch. This
      plan file is untracked at that point and survives the checkout.
      Push immediately and open the draft PR against `develop` per AGENTS.md §2.
      Its description uses `Source: #33` and `Status: WIP — do not review yet`;
      **do not put `Closes #33` in a draft PR description** — that auto-closes
      the issue on merge before the work is verified. `Closes #33` is added only
      at the Finish step, when converting to ready-for-review.

- [x] 1. **`audit.yml`: replace `rustsec/audit-check` with plain `cargo audit`** (item 1a).
      Drop the `rustsec/audit-check@v2.0.0` step and add three steps in its
      place, in this order:
      1. `dtolnay/rust-toolchain@stable`
      2. `cargo install cargo-audit --locked`
      3. `cargo audit --deny warnings`

      **All three steps carry `if: steps.lockfile.outputs.changed == 'true'`**
      — not just the last one. Gating only the final step would compile
      cargo-audit (~2–4 min) on every push and PR, including the majority that
      never audit anything.

      **Do not restructure this into a job-level `if:` or a second job.** The
      gate value is computed *inside* this job, and more importantly the
      `audit` job must keep *reporting success* when the gate is false. A job
      that is skipped outright produces a skipped check run, which is the exact
      hazard task 4 exists to avoid on the `ci.yml` side and would break #30's
      required-check configuration here.

      Keep `--locked` on the install — without it `cargo install` re-resolves
      dependencies on every run, which is non-reproducible and widens the trust
      surface for no gain. `--deny warnings` is required, not optional; see
      "Decisions taken during plan review".

      Do **not** substitute `taiki-e/install-action`: it trades one third-party
      action for another and reopens the pinning question.

- [x] 2. **`audit.yml`: add `workflow_dispatch:` with an explicit gate branch.**
      Confirmed in scope during plan review — implement it, do not skip it.
      Rationale: it provides an on-demand audit after a manual `cargo update`,
      which is exactly the step `AGENTS.md` prescribes but can only ask the
      maintainer to remember. That is its *whole* justification.

      **It is not a verification mechanism for this PR, and must not be treated
      as one.** GitHub only dispatches `workflow_dispatch` when the workflow
      file carrying that trigger already exists on the **default branch**
      (`main`). This PR targets `develop`, so the trigger stays inert until a
      later develop→main promotion lands it on `main`. Do not attempt
      `gh workflow run` on the feature branch — it will fail, and that failure
      is expected, not a bug in this task. Task 7 verifies the `cargo audit`
      path by a different route.

      Add `workflow_dispatch:` to the `on:` block. It takes no `inputs:` — do
      not add any. Do **not** add a `schedule:` trigger alongside it; see
      Out of scope.

      Then add a **first** branch to the `Cargo.lock` gate, before the existing
      `pull_request` test, setting `changed=true` with a comment explaining
      that a manual dispatch always audits. Write it in the **shell test syntax
      the existing gate already uses** — not GitHub expression syntax
      (`github.event_name == 'workflow_dispatch'`), which would not evaluate
      inside a `run:` block:
      ```sh
      if [ "${{ github.event_name }}" = "workflow_dispatch" ]; then
      ```
      This must be an **explicit** branch. Do not rely on the existing "no
      usable base commit" fallback to carry it: on a dispatch there is no
      `github.event.before`, so the fallback would set `changed=true` by
      accident. That happens to work, and a later refactor of the fallback
      would silently turn manual audits into no-ops. Keep the existing fallback
      in place for the `push`/`pull_request` paths; it is still correct there.

- [x] 3. **Both workflows: declare `permissions: contents: read`** (items 4a, 4b).
      Workflow level (top-level key, above `jobs:`) — not per-job. With task 1
      done this is all either workflow needs.
      For the implementer's understanding only, nothing to write down: this
      does *not* fix item 1. A workflow cannot request more than the fork-PR
      cap allows, so do not expect this task to change fork behaviour — the two
      are independent fixes that happen to touch adjacent lines.

- [x] 4. **`ci.yml`: add the `ci-success` aggregate job** (item 3a).
      The **job id must be the literal string `ci-success`** — that id is what
      becomes the check-run name, and #30 will require it by exact name. Do not
      use a different id with a `name:` override, and do not add a `name:` key
      at all; keep the mapping between id and check name direct and obvious.
      `needs: [fmt, clippy, test]` — three entries covering seven check runs,
      because a matrix job's `result` is the aggregate of all its legs, so
      `needs.clippy.result` is only `success` when all three OS legs succeeded.
      Do not try to enumerate the matrix legs individually in `needs:`; that
      reintroduces exactly the brittle name-coupling this job removes.
      Then `runs-on: ubuntu-latest`, and critically
      `if: always()` at job level — without it the job is *skipped* when a
      dependency fails, and a skipped required check does not block a merge.
      The job's single step must fail on any non-success result, not just
      `failure`:
      `if: contains(needs.*.result, 'failure') || contains(needs.*.result, 'cancelled') || contains(needs.*.result, 'skipped')` → `exit 1`.
      On the happy path that step is skipped and the job succeeds.
      This is what makes #30's required-check list two stable names
      (`ci-success`, `audit`) instead of eight matrix-expanded ones. `audit`
      stays separate because it lives in a different workflow file and a
      `needs:` aggregate cannot span workflows.

- [x] 5. **Both workflows: pin `actions/checkout`; comment the toolchain ref** (items 5a, 5b).
      Replace every `actions/checkout@v4` (3 in `ci.yml`, 1 in `audit.yml`)
      with
      `actions/checkout@11d5960a326750d5838078e36cf38b85af677262 # v4.4.0`.
      Leave every `dtolnay/rust-toolchain@stable` on the branch ref and record
      that this is deliberate, giving both reasons: the action infers which
      toolchain to install from `github.action_ref`, so a SHA ref breaks it
      unless `with: toolchain:` is also added; and #29 will move these lines to
      `@1.xx.0` refs anyway.
      Write that rationale **once per file, above the first
      `dtolnay/rust-toolchain` occurrence** — `ci.yml` has three (`fmt`,
      `clippy`, `test`) and `audit.yml` one after task 1; repeating a two-line
      justification at all four would be noise. Do not add a comment at the
      other occurrences.
      Note `ci-success` (task 4) needs neither a checkout nor a toolchain — it
      only inspects `needs.*.result`. Do not add either to it.
      Re-resolve the checkout SHA against
      `gh api repos/actions/checkout/git/ref/tags/v4` before committing, in
      case the tag has moved since this plan was written.

- [x] 6. **Both workflows: comment guarding `pull_request_target`** (item 6a).
      One line above each `on:` block: `pull_request` is deliberate and must
      not become `pull_request_target`. Say why in the comment — fork PRs run
      untrusted code, and `pull_request_target` would run it with a write token
      in the base repository's context. This is cheap insurance against a
      future change reintroducing it while chasing an unrelated permission
      error.

- [x] 7. **Verify against a live CI run** — the substantive verification step
      for this plan, since no Rust code changed.
      1. Confirm all jobs are green on the draft PR: `fmt`, `clippy` ×3,
         `test` ×3, `ci-success`, `audit`.
      2. Confirm `ci-success` appears as a check run under exactly that name.
      3. **Exercise the `cargo audit` path with a throwaway commit.** On the
         PR's own `pull_request` runs the audit steps will *skip*, because this
         PR does not touch `Cargo.lock` — that skip is correct, and it means a
         green `audit` check proves nothing about task 1. `workflow_dispatch`
         cannot be used here either (see task 2 — the trigger is inert until it
         reaches `main`).
         So: push a throwaway commit that hardcodes `changed=true` in the
         lockfile gate — replace the whole `if`/`elif`/`else` with a single
         `echo "changed=true" >> "$GITHUB_OUTPUT"` — and confirm the
         `dtolnay/rust-toolchain`, `cargo install cargo-audit --locked` and
         `cargo audit --deny warnings` steps all actually **run and pass, not
         skip**. Then force-drop that commit by the same procedure as sub-step
         4 below, and confirm the steps go back to skipping.
         Do **not** verify this by editing `Cargo.lock` — that is out of scope
         and would leave a real dependency change in the diff.
      4. **Verify the `ci-success` red path empirically.** Push a throwaway
         commit that breaks `cargo fmt --check` (mangled whitespace in any
         `src/*.rs` file — `cargo fmt` only inspects Rust sources, so there is
         no non-Rust way to trigger this), and confirm `ci-success` reports
         **failure** — not success, and not skipped. #30 depends on this
         behaviour being right; reading the YAML is not sufficient proof.

         Then **remove that commit from the branch entirely** — `git reset
         --hard` to the prior commit and `git push --force-with-lease`. Do
         **not** use `git revert`, which would leave both the breakage and its
         revert in the merged history. Force-pushing is permitted here because
         the PR is still a draft (`docs/git-workflow.md:81`); re-read that
         section before doing it. Confirm CI returns to green afterwards.

         This exercises the `failure` leg only. The `cancelled` and `skipped`
         legs are covered by construction in the `contains(...)` condition and
         are **not** separately exercised — that is an accepted limit of this
         verification, not an oversight to go chase.
      Record the observed run URLs in the PR description.

#### Task 7 observed results

| Sub-step | Run | Observed |
|---|---|---|
| 7.1 / 7.2 green path | [CI 31489461055](https://github.com/mrcmilano/mdv/actions/runs/31489461055), [Audit 31489461092](https://github.com/mrcmilano/mdv/actions/runs/31489461092) | All 9 check runs pass. `ci-success` appears under exactly that name; `audit` likewise. |
| Gate skips on this PR | [Audit 31488934058](https://github.com/mrcmilano/mdv/actions/runs/31488934058) | `dtolnay/rust-toolchain`, `Install cargo-audit`, `Run cargo audit` all **skipped** — correct, no `Cargo.lock` change, and confirms a green `audit` here proves nothing about task 1. |
| 7.3 audit path forced open | [Audit 31489014892](https://github.com/mrcmilano/mdv/actions/runs/31489014892) | All three steps **ran and passed**. `cargo audit --deny warnings` loaded 1211 advisories and scanned 42 crate dependencies clean. Commit force-dropped; steps confirmed skipping again. |
| 7.4 `ci-success` red path | [CI 31489352519](https://github.com/mrcmilano/mdv/actions/runs/31489352519) | `fmt: failure` → `ci-success: **failure**` — not success, not skipped. Commit force-dropped; CI back to green. |

The `cancelled` and `skipped` legs of the `contains(...)` condition were not
separately exercised — an accepted limit of this verification, per task 7.4.

### Finish
- [ ] Write / update tests for all implementation tasks above
      (N/A — no durable Rust source changes. Task 7's live CI run is the test;
      note this explicitly rather than silently skipping the checklist item.)
- [ ] Run full test suite — all tests pass
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected (expected: no change — see Impact assessment)
- [ ] Convert draft PR to ready-for-review; add `Closes #33` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 33 --remove-label agent --remove-label "in progress" --add-label needs-review`

---

## Follow-up

- **`actions/checkout` v4 → v7.** Out of scope here (see Out of scope), but v4
  runs on the Node 20 runtime that GitHub is phasing out. Worth an issue once
  this lands, so the deprecation is not discovered as a broken CI run.
- **#36** can proceed once this lands: the final action list is
  `actions/checkout` (SHA-pinned) and `dtolnay/rust-toolchain` (branch ref).
- **#30** can proceed once task 7 confirms `ci-success` behaves correctly on
  both the green and red paths.
- **#33's item 2 checkboxes (2a, 2b) will remain unticked.** They are cut, not
  forgotten — note this when closing the issue so the open checkboxes do not
  read as incomplete work.
