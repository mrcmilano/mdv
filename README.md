# mdv

A lean, read-only, interactive terminal Markdown viewer, written in Rust.

`mdv` opens a Markdown file and lets you scroll through it in your terminal —
no editor, no browser. It never writes to the file it opens.

> **Status:** Phases 1–4 are implemented — CLI parsing, terminal lifecycle,
> rendering of headings, paragraphs, emphasis/strong/strikethrough, inline
> code, links, images, horizontal rules, hard/soft breaks, code blocks,
> blockquotes (nested), ordered/unordered/task lists (nested), raw HTML
> passthrough, footnotes, and box-drawing tables (alignment, in-cell
> wrapping), with resize-triggered re-layout. Search and TOC land in a later
> phase; see `docs/mdv-build-plan.md` for the full spec and phase breakdown.

## Build

```bash
cargo build --release
```

## Usage

```
mdv <FILE>
mdv --help | -h
mdv --version | -V
```

## Keybindings (Normal mode)

| Key | Action |
|---|---|
| `j`, `↓` | scroll down 1 line |
| `k`, `↑` | scroll up 1 line |
| `d`, `PageDown`, `Space` | scroll down half a screen |
| `u`, `PageUp` | scroll up half a screen |
| `g`, `Home` | go to top |
| `G`, `End` | go to bottom |
| `q`, `Ctrl-C` | quit |

## Development

See `AGENTS.md` for the required workflow (Assess → Plan → Branch →
Implement → Finish) and `docs/mdv-build-plan.md` for the full specification.

```bash
cargo build                   # debug build
cargo test                    # all tests
cargo clippy -- -D warnings   # must be clean
cargo fmt                     # format
```
