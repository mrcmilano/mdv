# mdv

[![CI](https://github.com/mrcmilano/mdv/actions/workflows/ci.yml/badge.svg)](https://github.com/mrcmilano/mdv/actions/workflows/ci.yml)
[![Audit](https://github.com/mrcmilano/mdv/actions/workflows/audit.yml/badge.svg)](https://github.com/mrcmilano/mdv/actions/workflows/audit.yml)

A lean, read-only, interactive terminal Markdown viewer, written in Rust.

`mdv` opens a Markdown file and lets you scroll through it in your terminal —
no editor, no browser. It never writes to the file it opens.

## Features

- Headings, paragraphs, and inline formatting — bold, italic, strikethrough,
  inline code, links, and image placeholders.
- Fenced and indented code blocks, shown in a gutter with the language name.
- Nested blockquotes, and nested ordered, unordered, and task lists.
- Box-drawing tables with per-column alignment and in-cell wrapping.
- Footnotes collected at the end of the document, and raw HTML printed
  verbatim rather than interpreted.
- Text search: `/` opens the prompt, Enter runs the query, every match on
  screen is highlighted, and `n` / `N` cycle through them. Matching is
  case-insensitive.
- A table-of-contents overlay (`t`) to jump to any heading.
- Re-wraps the document to the new width when the terminal is resized.

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
| `t` | toggle TOC overlay |
| `/` | start search input |
| `n` / `N` | next / previous search match |
| `Esc` | close overlay / cancel search input / clear search highlights |
| `q`, `Ctrl-C` | quit |

`SearchInput` and `Toc` modes override these keys — see
`docs/mdv-build-plan.md` Section 6 for the full interaction model.

## Development

See `AGENTS.md` for the required workflow (Assess → Plan → Branch →
Implement → Finish) and `docs/mdv-build-plan.md` for the full specification.

```bash
cargo build                   # debug build
cargo test                    # all tests
cargo clippy -- -D warnings   # must be clean
cargo fmt                     # format
```
