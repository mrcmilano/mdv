# Architecture

`mdv` is a single Rust binary crate — no backend/frontend split, no shared
schemas. Full behavioral spec lives in `docs/mdv-build-plan.md`; this file
tracks the structural decisions as they're established during implementation.

## Module layout

```
src/
  main.rs      — arg parsing, terminal setup/teardown, event loop
  render.rs    — markdown events -> Document (styled, unwrapped blocks)
  layout.rs    — Document + terminal width -> Vec<Line> (wrapped, ready to print)
  view.rs      — viewport state: scroll offset, search state, TOC overlay
  input.rs     — key event -> Action enum
  style.rs     — Style struct and the fixed color palette
```

Do not add new top-level modules without updating this file and, if the
change is structural, `docs/mdv-build-plan.md` Section 4.

## Data flow

1. **Startup:** read file → `pulldown-cmark::Parser` → `render::build_document()`
   → `Document` (built once, never rebuilt).
2. **Layout:** `layout::wrap(&document, width)` → `Vec<Line>` plus
   `heading_lines: Vec<usize>`. Recomputed only on terminal resize.
3. **Event loop:** blocking `crossterm::event::read()`; each event maps to an
   `Action`; actions mutate `ViewState`; redraw only when state changed.
4. **Draw:** clear screen, print the visible line slice, then the status bar.
   One `crossterm::queue!`/flush pass per frame; nothing is printed outside
   the draw function.

## Boundaries

- `render.rs` never knows about terminal width or wrapping — that's `layout.rs`.
- `layout.rs` never knows about scroll position or input — that's `view.rs`.
- `input.rs` only maps raw key events to an `Action` enum; it never mutates
  state directly.
- `main.rs` owns terminal lifecycle (raw mode, alternate screen, panic hook)
  and wires the other modules together; it contains no rendering or layout
  logic itself.

See `docs/mdv-build-plan.md` Section 4 for the full data model
(`Style`, `Span`, `Line`, `Block`, `Document`, `TocEntry`) and Section 6 for
`ViewState`/`SearchState`.
