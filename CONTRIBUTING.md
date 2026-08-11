# Contributing to mdv

Thanks for your interest. `mdv` is a solo-maintained hobby project — issues and
pull requests are welcome, but there is no SLA on review turnaround. If
something sits for a while, it isn't being ignored.

## Reporting bugs and requesting features

Use the [GitHub issue tracker](https://github.com/mrcmilano/mdv/issues). For a
rendering bug, a small Markdown snippet that reproduces it is the most useful
thing you can include.

## Pull requests

Target `develop`, not `main` — and say so explicitly. `main` is this repo's
default branch, so both `gh pr create` and the GitHub web UI preselect it;
pass `--base develop` (or change the base in the UI) or your PR opens against
the wrong branch.

Before opening a PR, make sure these pass:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

CI runs clippy and the test suite with `--locked`, so if your change adds,
removes, or bumps a dependency you must commit the updated `Cargo.lock` —
otherwise CI fails even though everything passes locally. This is the one
non-obvious way a well-formed PR goes red.

The other one is CI that never starts: workflows on a PR from a fork require
maintainer approval before they run at all. A fresh PR showing no checks — or
"waiting for approval" — is expected, not a broken pipeline. It clears once the
run is approved.

See `AGENTS.md` for build/test/lint commands and code style. Note: the
plan-file/label workflow documented there is the maintainer's own process —
external PRs don't need to follow it.

## Scope

`mdv` is deliberately a lean, read-only viewer. Some things are non-goals by
design rather than missing work — editing, image rendering, HTML rendering,
config files, themes, and plugins are all listed under `docs/mdv-build-plan.md`
Section 1, "Goals and non-goals". A feature on that list will be declined
however well implemented.

Size counts too, not just the non-goals list: #13 (Mermaid diagram rendering)
was assessed and planned in full, then declined — a flowchart layout engine is
more machinery than a lean viewer should carry.

If you're unsure whether an idea fits, open an issue before writing the code.

## Dependencies

The project has exactly three direct runtime dependencies (`pulldown-cmark`,
`crossterm`, `unicode-width`), and keeping it that way is a design constraint,
not an accident. Please discuss in an issue first before adding a fourth.
