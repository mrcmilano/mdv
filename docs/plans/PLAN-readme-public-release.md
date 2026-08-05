# PLAN: README public release wording

_Branch:_ `chore/issue-24-readme-public-release`
_Date:_ 2026-08-04
_Status:_ IN PROGRESS
_Source:_ #24
_PR:_ #26

---

## Problem

`README.md` still reads as an internal build-tracking document rather than
public-facing documentation. The `> **Status:** Phases 1–5 are implemented — …`
blockquote enumerates build phases, declares Phase 6 "out of scope for v1", and
points at the internal build plan — none of which tells a visitor what the tool
does or how to use it. Alongside that, the README omits several things a public
Rust repo is normally expected to state: how to actually install the binary
(not just build it), which platforms are supported, that the project is MIT
licensed, and that there is a known open rendering bug. The `## Development`
section links inward to `AGENTS.md`, whose plan-file/label ritual is the
maintainer's own process and would read to an outside contributor as a set of
requirements imposed on them.

This plan rewrites `README.md` for a public audience and adds a minimal
`CONTRIBUTING.md` that absorbs the contributor-facing content, so the public
README never needs to link to `AGENTS.md` directly.

## Out of scope

- **Everything tracked in #20** — repo visibility flip, `main`/`develop` branch
  state, CI clippy `--all-targets`, removal of `docs/api-error-handling.md`, and
  the `AGENTS.md` universal/maintainer-specific split. This plan has no
  execution-order dependency on #20 (see Impact assessment).
- **Publishing to crates.io.** `cargo install --path .` is the install path this
  plan ships. Reserving the crate name, publishing, and setting a version policy
  is separate work — a follow-up issue is filed in task 9.
- **Screenshot / asciicast / GIF.** Capturing a real terminal session needs a
  human at a terminal choosing a sample file and window size, and commits a
  binary asset. A hand-written imitation of mdv's output was rejected: it isn't
  produced by the real renderer, so it rots silently the moment layout changes,
  and a fake render is a bad advertisement for a rendering tool. Follow-up issue
  filed in task 9.
- **MSRV / `rust-version` in `Cargo.toml`.** An MSRV is a promise CI enforces;
  stating an unverified number is worse than stating nothing. Deliberately
  omitted here — it needs its own issue (filed in task 9) covering measurement,
  the `Cargo.toml` field, and a pinned CI job, all of which are outside this
  plan's README/CONTRIBUTING scope.

  Measured floor as of 2026-08-04, for that issue's benefit: **1.81.0**, set by
  `derive_more` 2.1.1 — a *transitive* dep pulled in by `crossterm`'s default
  features (`default = [… "derive-more"]`; `Cargo.toml` does not set
  `default-features = false`). None of the three direct deps bind:
  `pulldown-cmark` 0.13.4 is 1.71.1, `unicode-width` 0.2.2 is 1.66,
  `crossterm` 0.29.0 is 1.63.0. This corrects an earlier draft of this plan that
  named the direct deps as the floor — the real constraint was two releases
  higher and came from a crate not named anywhere in the project. Treat 1.81.0
  as a lower bound to verify by building, not as a number to publish unverified.

  **Assessed as not blocking the public flip** (#20): mdv is a binary, so no
  downstream resolver consults its `rust-version`, and the worst case for a user
  on an old toolchain is a recoverable build error. Task 3's `--locked` flag
  removes most of the exposure independently.
- **Fixing #14.** The README discloses it; it does not fix it.
- **Any change to `src/`, `Cargo.toml`, or tests.** This plan touches
  `README.md` and `CONTRIBUTING.md` only.
- **`## Usage` and `## Keybindings` content.** Verified in #24 against
  `src/main.rs` (`USAGE` const, `Cli::Help` / `Cli::Version`) as matching actual
  CLI behaviour exactly. Left as-is.

## Impact assessment

**Files changed**

| File | Change |
|---|---|
| `README.md` | Status blockquote removed. New sections: Features, Known issues, License. Rewritten: Build → Install (incl. platform line), Development. Usage + Keybindings untouched. |
| `CONTRIBUTING.md` | New file at repo root. |
| `docs/plans/PLAN-readme-public-release.md` | This plan. |

No code, no tests, no dependency changes. `cargo test` / `cargo clippy` results
are unaffected by construction, but the baseline is still confirmed in the
Finish checklist.

**Branch base — needs care.** The worktree this plan was written in
(`/Users/marco/orca/workspaces/mdv/remove-dev-wording-from-readme`) is currently
on branch `mrcmilano/remove-dev-wording-from-readme`, which sits on the `init`
commit and contains **no `README.md` at all**. The new branch must be cut from
`develop`, not from the current HEAD. Because `develop` is checked out in
another worktree, git will refuse a plain `git checkout develop` here; use the
start-point form, which does not check `develop` out:

```bash
git fetch origin
git checkout -b chore/issue-24-readme-public-release origin/develop
```

Confirm `README.md` exists and matches `develop` before editing anything.
This plan file is untracked and survives the checkout.

**Relationship to #20.** `CONTRIBUTING.md` refers to `AGENTS.md` generically —
no link to a specific section or anchor — so it stays accurate whether or not
#20's `AGENTS.md` split has landed. The public README gains no link to
`AGENTS.md`, so #20 can land before, after, or independently of this work with
no rework here.

**Accuracy risks this plan actively guards against**

- Platform claim must be backed by what CI actually runs — verified in task 4
  against `.github/workflows/ci.yml`, not assumed.
- #22 (UTF-8 BOM) was **closed** by `14faaa4`, after #24 was written. The Known
  issues section must therefore reference **#14 only**. Issue state is
  re-checked at implementation time (task 5) in case #14 also closes.
- #14's own body states the defect is not yet characterised ("Needs its own
  impact assessment … before scoping a fix"). The README must describe it as a
  reported issue with a link, not assert a specific broken behaviour.

## Open questions

Resolved in conversation before this plan was written — recorded here for the
implementation record:

- **Install path** → `cargo install --path .`, build-from-source only. Not
  crates.io (separate work).
- **Screenshot/demo** → omitted from this issue; follow-up issue instead.
- **MSRV** → omitted; not stated in README or `Cargo.toml`.
- **License** → one-line `## License` section, not a third badge (two CI badges
  already head the file).
- **Feature list** → the Status blockquote is the README's only inventory of
  what mdv renders. Deleting it outright would leave a public README that never
  states the tool does tables, search, or a TOC. Its *content* is worth keeping;
  only its *framing* (phase numbers, "out of scope for v1", build-plan pointer)
  is the problem. Resolution: a short `## Features` section (task 2) carrying
  the capabilities as capabilities. This is a deliberate addition to #24's
  literal ask, made because "remove the blockquote" and "produce a good public
  README" pull in opposite directions here.

---

## Tasks

### Process notes

These carry instructions that live in the plan template's HTML comments and are
easy to lose once the template is filled in:

- **Plan status flips:** set `_Status:_` `DRAFT` → `APPROVED` *before* the task-0
  commit. Set it to `IN PROGRESS` on the first content-task commit (task 1) and
  commit the updated plan alongside that task's changes. Set it to `READY` in
  the Finish checklist.
- **Draft PR:** task 0's plan-file commit is the first push. Immediately after
  pushing, open a draft PR targeting `develop` with the AGENTS.md §2 header
  block (`Plan:` / `Source: #24` / `Status: WIP — do not review yet`) plus the
  `## What` / `## Why` / `## Notes for reviewer` sections from
  docs/git-workflow.md. Use `ref #24`, **not** `Closes #24`, while it is a draft.
- **`_PR:_` header:** once the draft PR exists, fill `_PR:_ #<n>` into this
  plan's header and commit it (precedent: `16fdb45`).
- **One commit per task.** Tasks 1–7 all touch `README.md`; they are still
  seven separate commits, one per task, subject
  `docs: <description> (task <N>, ref #24)`. Push after every 2 completed tasks.

### Implementation

- [x] 0. Create branch `chore/issue-24-readme-public-release` from develop following docs/git-workflow.md
- [x] 1. Remove the `> **Status:** Phases 1–5 are implemented — …` blockquote from
      `README.md` in full. **Remove that blockquote and nothing else in this
      task** — the two intro paragraphs above it (`A lean, read-only…` and
      ``mdv` opens a Markdown file…`) stay verbatim; they already carry both what
      the tool is and the never-writes guarantee. The feature inventory the
      blockquote carried is re-added as its own section in task 2, so this task
      leaves the README temporarily thinner — that is expected, not a gap to
      patch here.
- [x] 2. Add a `## Features` section between the intro and the build/install
      section (still titled `## Build` at this point — task 3 renames it), carrying
      the capability inventory that task 1 removed, rewritten as capabilities
      rather than build history. Source the list from the deleted blockquote and
      verify each claim against `src/render.rs` / `src/view.rs` before writing it.
      Roughly 5–8 bullets covering: headings/paragraphs/inline formatting; code
      blocks; blockquotes and nested lists incl. task lists; box-drawing tables
      with alignment and in-cell wrapping; footnotes and raw-HTML passthrough;
      incremental search with match highlighting; TOC overlay; resize-aware
      re-layout.
      **Hard constraints on wording:** no phase numbers, no "Phase 6", no "v1",
      no "implemented"/"complete" framing, and **no claim of full CommonMark
      coverage** — the Known issues section (task 5) exists precisely because
      that claim would be false. Describe what it does, not how much of a spec
      it covers.
      **Deviation from this task's suggested wording, recorded for the review
      record:** the bullet list does **not** call the search "incremental".
      Verification against `src/view.rs` (`start_search` / `search_push_char` /
      `execute_search`), `src/input.rs` (`KeyCode::Enter => Action::SearchExecute`)
      and build plan Section 6 "Search semantics" ("On Enter: collect matching
      line indices") shows the query is buffered while typing and only runs on
      Enter — nothing is matched or highlighted per keystroke. The blockquote's
      "incremental" was inaccurate; written as what it actually does instead,
      per this task's own instruction to verify each claim before writing it.
- [x] 3. Replace `## Build` with `## Install`, directly above `## Usage`. State
      that there are no published binaries yet and mdv is built from source
      (Rust toolchain required). Give the full sequence a visitor can paste —
      `git clone https://github.com/mrcmilano/mdv.git`, `cd mdv`, then
      `cargo install --locked --path .` to get `mdv` on `PATH` — and keep
      `cargo build --locked --release` as the alternative for anyone who just
      wants the binary at `target/release/mdv`.
      **`--locked` is required, not decorative.** `cargo install` ignores
      `Cargo.lock` by default (including with `--path`) and re-resolves to the
      newest semver-compatible versions, so without it a visitor's very first
      command builds a dependency tree nobody has tested and can fail on a
      transitive crate's toolchain requirement. CI already runs
      `cargo test --locked` / `cargo clippy --locked`; the README must match what
      CI actually verifies.
- [x] 4. Add a one-line platform statement to the Install section. **Read
      `.github/workflows/ci.yml` first** and name exactly the operating systems
      its matrix runs — do not copy "Linux, macOS, and Windows" from #24 without
      confirming. Phrase it as **"Tested on …"**, not "Supports …": CI proves the
      former, and nothing in the repo backs the latter.
- [x] 5. Add a short `## Known issues` section after `## Keybindings`. **Run
      `gh issue list --state open --label bug` first** and list only what is
      still open at that moment; today that is #14 alone (#22 closed in
      `14faaa4`). Describe #14 as a reported issue with `text`-language fenced
      code blocks and link it — do not assert a specific defect, since #14's own
      body says the behaviour is not yet characterised. Close the section with a
      pointer to the issue tracker.
      **If that command returns nothing** (i.e. #14 has closed too), keep the
      section and reduce it to the tracker pointer alone — do not drop it and do
      not invent a limitation to fill it.
- [x] 6. Add a `## License` section as the **last section of the file**, below
      `## Development`: one line, MIT, linking the existing `LICENSE` file.
      No badge.
- [ ] 7. Rewrite `## Development` so its **entire** final content is: the four
      cargo commands already there (`cargo build`, `cargo test`,
      `cargo clippy -- -D warnings`, `cargo fmt`) in their existing fenced block,
      preceded by one line pointing anyone who wants to contribute at
      `CONTRIBUTING.md`. The sentence *"See `AGENTS.md` for the required workflow
      (Assess → Plan → Branch → Implement → Finish) and `docs/mdv-build-plan.md`
      for the full specification."* is deleted whole — both links go, and the
      build-plan pointer is **not** reinstated elsewhere in this section.
      **Decided, do not re-litigate:** the separate `docs/mdv-build-plan.md`
      pointer under `## Keybindings` **stays** — the build plan is a legitimate
      public spec document and #24 raised no objection to it. `AGENTS.md` must
      not be linked from `README.md` at all once this task is done; verify with
      `grep -n 'AGENTS' README.md` returning nothing.
- [ ] 8. Create `CONTRIBUTING.md` at repo root, sized to a solo-maintained hobby
      project — no code of conduct, no governance model, no contributor ladder.
      Content, per #24's Decisions:
      - Expectation-setting: solo-maintained hobby project, no SLA on review
        turnaround.
      - Where to file issues (GitHub issues).
      - PR bar: target `develop`; `cargo fmt --check`,
        `cargo clippy -- -D warnings`, `cargo test` must pass. Add a short note
        that CI runs clippy and test with `--locked`, so a PR that changes
        dependencies must commit the updated `Cargo.lock` or CI fails — this is
        the one non-obvious way a well-formed external PR goes red.
      - Verbatim-in-spirit: *"See `AGENTS.md` for build/test/lint commands and
        code style. Note: the plan-file/label workflow documented there is the
        maintainer's own process — external PRs don't need to follow it."*
        Keep the `AGENTS.md` reference **generic** — plain-text filename, no
        section link and no `#anchor`, so it survives #20's AGENTS.md split
        whenever that lands.
      - Scope: intentionally a lean/minimal viewer by design. Reference the
        build plan's non-goals as plain text — `docs/mdv-build-plan.md`
        Section 1, "Goals and non-goals" — again with no `#anchor`. Precedent:
        #13 (Mermaid) declined as out of scope.
      - Do not add dependencies beyond the three named in `AGENTS.md` /
        the build plan without discussing first.
- [ ] 9. File three follow-up issues, no milestone (milestones are human-only per
      AGENTS.md §5) and **no `agent` label** — assigning work to the agent is the
      maintainer's call. Each references #24:
      - "Add a screenshot or asciicast to the README" — label `documentation`.
        Explain why it was split out: needs a human at a real terminal choosing
        a sample file and window size, and commits a binary asset.
      - "Publish mdv to crates.io" — label `enhancement`. Name reservation,
        publish, version policy; note that README's Install section switches to
        `cargo install mdv` once it lands.
      - "Declare and enforce an MSRV" — label `enhancement`. Measure the real
        floor by building against candidate toolchains (lower bound 1.81.0 as of
        2026-08-04, set by the transitive `derive_more` 2.1.1, *not* by any
        direct dep — see this plan's Out of scope section), add `rust-version` to
        `Cargo.toml`, and add a CI job pinned to that toolchain so the claim is
        enforced rather than asserted. Note in the issue body that this was
        assessed as **not blocking** the public flip (#20), and why: mdv is a
        binary, so no downstream resolver reads its `rust-version`.

### Verification

- [ ] 10. Verify every command the README now claims actually works — run them
      **exactly as written in the README, `--locked` included**, since the point
      is to verify the visitor's experience, not an approximation of it. Run
      `cargo build --locked --release`, then install to a throwaway root so the
      user's toolchain is untouched:
      `cargo install --locked --path . --root "$SCRATCH"` where `$SCRATCH` is a
      temp dir. Do **not** run it without `--root`, which writes to
      `~/.cargo/bin`. Confirm `"$SCRATCH"/bin/mdv --version` runs and prints the
      version. If `--locked` fails, `Cargo.lock` is stale against `Cargo.toml` —
      stop and report rather than dropping the flag to make it pass.
- [ ] 11. Dogfood, using the binary from task 10: view `README.md` and
      `CONTRIBUTING.md` in mdv and confirm both render sanely — headings, the
      keybindings table, the new Features bullets, and the fenced blocks. This is
      the one rendering check that matters for a doc whose whole job is to be
      read in the tool it documents.
      **This task usually cannot be completed by an agent.** `src/main.rs`
      (`ensure_stdout_is_tty`, ~line 136) rejects a non-TTY stdout with
      `mdv: interactive viewer requires a terminal`, and an agent shell is not a
      TTY. A PTY wrapper (`script -q /dev/null …`) will get past that check but
      returns a full-screen alternate-buffer capture that is not reliably
      readable. **If the check cannot actually be run: leave this task
      unchecked, say so explicitly in the PR's "Notes for reviewer" as a manual
      step for the maintainer, and do not mark it done.** Reporting it as passed
      without having seen the output is the failure mode this note exists to
      prevent.
      If a rendering defect *is* observed, file it as a new issue rather than
      fixing it here (out of scope).
- [ ] 12. Re-read the final `README.md` end to end against #24's checklist: no
      phase numbers, no "v1", no build-plan-as-status framing, no `AGENTS.md`
      link, no CommonMark-coverage claim, no unverified claims. Confirm section
      order reads: badges → intro → Features → Install → Usage → Keybindings →
      Known issues → Development → License.

### Finish

- [ ] Write / update tests for all implementation tasks above
      — **N/A: documentation-only change, no code touched.** Recorded here
      rather than silently skipped; tasks 10–12 are the substitute verification.
- [ ] Run full test suite — all tests pass (baseline confirmation only)
- [ ] Run `/skill:adversarial-review` — resolve all FIX REQUIRED findings before proceeding
      (FIX REQUIRED: add tasks to Implementation above and complete them;
       LOW: document rationale in Deferred findings section below)
- [ ] Update `README.md` if affected — **N/A: `README.md` is the deliverable.**
- [ ] Convert draft PR to ready-for-review; add `Closes #24` to PR description;
      set this plan's `_Status:_` to `READY`
- [ ] Remove `agent` and `in progress` labels; add `needs-review` label on source issue
      `gh issue edit 24 --remove-label agent --remove-label "in progress" --add-label needs-review`
