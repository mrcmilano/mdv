# Build Plan: `mdv` — Terminal Markdown Viewer

A lean, read-only, interactive terminal viewer for Markdown files, written in Rust.
This document is the complete specification. Build it phase by phase; each phase has
explicit acceptance criteria. Do not add features not listed here.

---

## 1. Goals and non-goals

**Goals**

- Fully render a Markdown file in the terminal with clear visual structure, replacing
  the need for VS Code's preview during visual inspection.
- Interactive TUI: scrolling, text search, jump-to-heading via a table of contents.
- Lean: exactly 3 direct runtime dependencies (transitive deps they pull are
  acceptable), fast startup (< 50 ms perceived on files up
  to 1 MB), single static binary.

**Non-goals (do not implement)**

- Editing of any kind. The tool never writes to the file.
- Rendering images (render alt text placeholder instead).
- Parsing/rendering raw HTML (print it dimmed, verbatim).
- Syntax highlighting inside code blocks in v1 (see Phase 6, optional).
- Config files, themes, or plugins. One built-in style.
- Live reload / file watching in v1 (see Stretch goals).

---

## 2. Dependencies and project setup

```toml
[package]
name = "mdv"
version = "0.1.0"
edition = "2021"

[dependencies]
pulldown-cmark = { version = "0.13", default-features = false }
crossterm = "0.29"
unicode-width = "0.2"

[profile.release]
lto = true
strip = true
```

Rules:

- `pulldown-cmark` with `default-features = false` (we don't need SIMD or serde).
  Enable parser `Options`: `ENABLE_TABLES`, `ENABLE_STRIKETHROUGH`,
  `ENABLE_TASKLISTS`, `ENABLE_FOOTNOTES`.
- No `clap`, no `anyhow`, no `thiserror`, no `ratatui`, no async runtime.
  CLI parsing and error types are hand-rolled (they are trivial here).
- If a version above doesn't exist when building, use the latest stable release of
  that crate and adapt to its API; do not add extra crates to compensate.

---

## 3. CLI behavior

```
mdv <FILE>
mdv --help | -h
mdv --version | -V
```

- Exactly one positional argument: path to a Markdown file.
- No argument, unknown flag, unreadable file, or file that isn't valid UTF-8:
  print a one-line error to stderr and exit with code 1. Example:
  `mdv: cannot read 'notes.md': No such file or directory`.
- `--help` prints usage plus the keybinding table (Section 7) to stdout, exit 0.
- `--version` prints `mdv <version>` using `env!("CARGO_PKG_VERSION")`, exit 0.
- If stdout is not a TTY (e.g. output is piped), print an error
  `mdv: interactive viewer requires a terminal` and exit 1. (Static rendering to
  pipes is explicitly out of scope for v1.)

---

## 4. Architecture

Module layout (all in one binary crate):

```
src/
  main.rs      — arg parsing, terminal setup/teardown, event loop
  render.rs    — markdown events -> Document (styled, unwrapped blocks)
  layout.rs    — Document + terminal width -> Vec<Line> (wrapped, ready to print)
  view.rs      — viewport state: scroll offset, search state, TOC overlay
  input.rs     — key event -> Action enum
  style.rs     — Style struct and the fixed color palette
```

### Core data model

```rust
// style.rs
#[derive(Clone, Copy, PartialEq, Default)]
struct Style {
    fg: Option<Color>,        // crossterm::style::Color
    bold: bool,
    italic: bool,
    dim: bool,
    strikethrough: bool,
    reverse: bool,            // used for search-match highlight
    underline: bool,          // used for link text (Section 5)
}

// A run of text with one style.
struct Span { text: String, style: Style }

// One visual line after wrapping. Rendering = print spans left to right.
struct Line { spans: Vec<Span> }

// render.rs output: block-level structure BEFORE wrapping.
enum Block {
    Heading { level: u8, spans: Vec<Span> },
    Paragraph { spans: Vec<Span> },
    CodeBlock { language: Option<String>, lines: Vec<String> },
    BlockQuote { blocks: Vec<Block> },           // nested
    List { ordered: Option<u64>, items: Vec<Vec<Block>> },
    Table { header: Vec<Vec<Span>>, rows: Vec<Vec<Vec<Span>>>, alignments: Vec<Alignment> },
    Rule,
    Html { lines: Vec<String> },
    FootnoteDef { label: String, blocks: Vec<Block> },
}

struct Document {
    blocks: Vec<Block>,
    headings: Vec<TocEntry>,   // filled during render
}

struct TocEntry { level: u8, text: String, block_index: usize }
```

### Data flow

1. **Startup:** read file → `pulldown-cmark::Parser` → `render::build_document()`
   → `Document` (built once, never rebuilt).
2. **Layout:** `layout::wrap(&document, width)` → `Vec<Line>` plus
   `heading_lines: Vec<usize>` mapping each `TocEntry` to its first wrapped line
   index. Recomputed only on terminal resize.
3. **Event loop:** blocking `crossterm::event::read()`; each event maps to an
   `Action`; actions mutate `ViewState`; redraw only when state changed.
4. **Draw:** clear screen, print visible slice `lines[offset .. offset + height - 1]`,
   then a 1-row status bar (Section 8). Use `crossterm::queue!` and flush once per
   frame; never print outside the draw function.

### Terminal lifecycle (must be robust)

- On start: enter alternate screen, enable raw mode, hide cursor.
- On exit (any path, including panic): restore. Install a panic hook that restores
  the terminal *before* printing the panic message, then chain to the previous hook.
- Handle `Event::Resize(w, h)`: re-run layout, clamp scroll offset, redraw.

---

## 5. Rendering specification

Fixed palette. Use only crossterm's 16 ANSI colors so the user's terminal theme
applies (do not use RGB values). All output must be wrapped to
`content_width = terminal_width.min(100)` and left-aligned at column 0; if the
terminal is wider than 100 columns, cap content at 100 (long lines are unreadable).

Element-by-element rules:

| Element | Rendering |
|---|---|
| H1 | Bold + Cyan, uppercase the text (all inline content, including code spans), blank line before and after, followed by a `═` underline the width of the text |
| H2 | Bold + Cyan, followed by a `─` underline the width of the text |
| H3 | Bold + Cyan |
| H4–H6 | Bold, prefixed with `§ ` |
| Paragraph | Plain text, wrapped; blank line between blocks |
| **bold** | bold |
| *italic* | italic |
| ~~strike~~ | strikethrough |
| `inline code` | Yellow, no background |
| Code block | Each line prefixed with 2-space indent + Dim `│ ` gutter; content in Yellow; language name (if any) shown Dim right after the opening, as `│ (rust)` on its own line. No wrapping: lines longer than content width are truncated with a Dim `…`. **Deliberate decision:** truncated overflow is not reachable by `/` search (search sees rendered lines only); this is accepted, do not change it to wrapping |
| Blockquote | Prefix every wrapped line with Dim Green `┃ `; nested quotes stack prefixes (`┃ ┃ `); quoted text Dim |
| Unordered list | Bullet `•` for level 0, `◦` level 1, `▪` level 2+; 2-space indent per level; continuation lines align under the text, not the bullet |
| Ordered list | `1.` `2.` … starting from the source's start number; same indent rules |
| Task list | `[✓]` in Green for checked, `[ ]` for unchecked, before the item text |
| Table | Unicode box drawing (`┌ ┬ ┐ ├ ┼ ┤ └ ┴ ┘ │ ─`), header row Bold, respect column alignments. Column widths = max cell display-width (via `unicode-width`), but if the total exceeds content width, shrink the widest columns and wrap cell text inside cells |
| Link `[text](url)` | text in Blue underlined, followed by Dim ` (url)`. Skip the `(url)` suffix when url == text (autolinks) |
| Image `![alt](src)` | Dim `[image: alt]` |
| Horizontal rule | A full-content-width Dim `─` line |
| Footnote ref | Dim `[^label]` inline; footnote definitions rendered at the very end under a rule, as `[^label]: ` + content |
| Raw HTML | Verbatim, Dim |
| Hard break | New line within the paragraph |
| Soft break | Treated as a space (paragraph reflows) |

Wrapping rules (`layout.rs`):

- Break at spaces; a single word longer than the content width is hard-broken at
  the width boundary.
- All width calculations use `unicode_width::UnicodeWidthStr` (never `str::len()`
  or `chars().count()`), so CJK and emoji align correctly.
- Style must survive wrapping: a bold span split across two lines is bold on both.
- **Sanitization (security-critical, applied to ALL source text before layout):**
  strip `\r`; replace each tab with four literal spaces inside code blocks and
  with a single space elsewhere (no tab-stop math);
  replace every other C0 control character (U+0000–U+001F), DEL (U+007F), and C1
  controls (U+0080–U+009F) with `�` (U+FFFD). Consequence: the only ANSI/OSC
  escape sequences ever written to the terminal are the ones mdv generates
  itself. There is no exception for code blocks or raw HTML.
- **Prefix composition:** when blocks nest, per-line prefixes compose
  outermost-first — e.g. a code block inside a blockquote renders as
  `┃   │ code`. The inner block's content width shrinks by the total prefix
  width, clamped to a minimum of 1 column. At absurdly narrow terminal widths
  output may degrade visually but must never panic or underflow.

---

## 6. Interaction model

### ViewState

```rust
struct ViewState {
    offset: usize,                 // index of first visible Line
    lines: Vec<Line>,              // current layout
    heading_lines: Vec<usize>,     // TOC target lines
    mode: Mode,                    // Normal | SearchInput | Toc
    search: Option<SearchState>,   // last executed search
    toc_cursor: usize,
}

struct SearchState { query: String, matches: Vec<usize> /* line indices */, current: usize }
```

### Modes

- **Normal** — scrolling and shortcuts active.
- **SearchInput** — `/` was pressed; status bar becomes an input line
  (`/query▌`); printable chars append, Backspace deletes, Enter executes,
  Esc cancels back to Normal.
- **Toc** — centered overlay box listing all headings, indented 2 spaces per
  level below H1, current selection rendered Reverse. Geometry: width =
  `min(60, terminal_width - 4)`, height = `min(heading_count + 2,
  terminal_height - 4)`, single-line border (`┌─┐│└┘`), title ` Contents ` in
  the top border. Heading text wider than the box is truncated with a trailing
  `…`. If there are more headings than visible rows, the list scrolls to keep
  the selection visible. `j/k`/arrows move, Enter jumps (scroll so the heading
  is the top visible line), Esc/`t` closes. If the document has no headings,
  `t` does not open an overlay; it shows `No headings` in the status bar until
  the next keypress.

### Search semantics

- Case-insensitive substring match against each `Line`'s concatenated plain text.
- On Enter: collect matching line indices; jump to the first match at or after the
  current offset (wrapping to the top if none). `n` / `N` cycle forward/backward.
- All matches on visible lines are highlighted with `reverse` style at the exact
  matched character ranges (map plain-text ranges back into spans, splitting spans
  as needed).
- No matches: status bar shows `Pattern not found: query` until the next keypress.
- Search state persists (highlights + n/N) until a new search or Esc in Normal mode
  clears it.

---

## 7. Keybindings (exact, complete — Normal mode)

These bindings apply in Normal mode. SearchInput and Toc modes override keys as
defined in Section 6 (e.g. `q` is a literal character while typing a search).

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

Scrolling is clamped to `max_offset = lines.len().saturating_sub(viewport_height)`
(never negative), where `viewport_height = terminal_height - 1`. `G`/`End` set
`offset = max_offset`, i.e. the last screenful fills the viewport — you cannot
scroll past the end. This is the same `max_offset` that equals 100% / `Bot` in
the Section 8 formula.
Any key not listed is ignored silently.

---

## 8. Status bar

One reversed-video row pinned to the bottom:

```
 notes.md                                    42% · 120/284 · t:toc /:search q:quit
```

- Left: file name (basename).
- Right: scroll percentage (`Top` at 0, `Bot` when the last line is visible),
  `current_top_line/total_lines`, and the hint `t:toc /:search q:quit`.
  Percentage formula: `offset * 100 / max(1, lines.len().saturating_sub(viewport_height))`,
  clamped to 0–100, where `viewport_height = terminal_height - 1` (status bar row).
- In SearchInput mode the entire bar is replaced by the `/query▌` input line.

---

## 9. Build phases and acceptance criteria

Work strictly in this order. After each phase, `cargo build` must succeed with no
warnings (`cargo clippy -- -D warnings` clean) and the listed criteria must pass.

**Phase 1 — Skeleton and terminal lifecycle.**
CLI parsing, file loading, alternate screen + raw mode in/out, panic hook, event
loop that draws the raw file text with j/k/g/G/q working.
*Accept:* open a file, scroll it, quit; `Ctrl-C` and an injected `panic!()` both
leave the terminal in a sane state; piping stdout errors out per Section 3.

**Phase 2 — Rendering pipeline for inline + basic blocks.**
`render.rs` + `layout.rs` for headings, paragraphs, emphasis/strong/strike, inline
code, links, images, rules, hard/soft breaks. Resize re-layout.
*Accept:* the test corpus file (Section 10) shows correct styling; resizing the
terminal reflows paragraphs; a bold word split across a wrap stays bold.

**Phase 3 — Structural blocks.**
Code blocks, blockquotes (nested), ordered/unordered/task lists (nested),
raw HTML passthrough, footnotes.
*Accept:* a 3-level nested list aligns continuation lines correctly; nested
blockquotes stack gutters; code block lines never wrap.

**Phase 4 — Tables.**
Box-drawing tables with alignment and in-cell wrapping when too wide.
*Accept:* a table wider than the terminal shrinks/wraps without panicking at
width 40; alignment markers (`:---:` etc.) are respected.

**Phase 5 — Search and TOC.**
Full Section 6 behavior.
*Accept:* search for a string that appears inside a bold span highlights exactly
the matched characters; `n` wraps around; TOC jump lands the heading at the top
of the viewport.

**Phase 6 (optional, only if explicitly requested later) — Syntax highlighting.**
Behind a cargo feature `highlight` adding `syntect` with the `default-fancy`
feature set (pure Rust, no onig). Off by default. Not part of v1 acceptance.

---

## 10. Testing

- **Unit tests** (in-module `#[cfg(test)]`):
  - `layout`: wrapping at width boundaries, unicode widths (test with `你好` and
    emoji), style preservation across wraps, single overlong word hard break.
  - `render`: markdown snippet → expected `Block` structure for each element type.
  - Table column-width computation, including the shrink path.
  - Search range-to-span mapping (match spanning two spans).
  - The two security tests defined in Section 12 (ESC-byte sanitization,
    no-panic on adversarial input at widths 1/2/40).
- **Snapshot test:** render the corpus file at width 80 into plain text with style
  markers (e.g. `**bold**` re-serialized) and compare against a checked-in
  `tests/snapshots/corpus.txt`. No snapshot crate — a plain string compare with a
  helper that rewrites the file when `UPDATE_SNAPSHOTS=1` is set.
- **Corpus file** `tests/corpus.md`: create it containing at least one instance of
  every element in Section 5, including a 3-level nested list, nested blockquote,
  a wide table, CJK text, emoji, a 200-char unbroken word, task lists, and footnotes.
- No integration tests that spawn a real TTY (not worth the machinery).

---

## 11. Error handling and performance

- Library-boundary errors (`io::Error`) are handled at `main` and become the
  one-line stderr message. Internal invariant violations may panic (the hook
  restores the terminal).
- The whole file is read and rendered eagerly at startup. This is acceptable:
  target files are documentation-sized (≤ a few MB). Do not implement lazy
  parsing or virtual scrolling.
- Redraw only on state change; draw only the visible slice; one flush per frame.

---

## 12. Security posture

Threat model: the only untrusted input is the Markdown file itself. The program
runs locally, never touches the network, never writes to any file, reads no
environment variables, and spawns no subprocesses. Enforce these invariants:

- `#![forbid(unsafe_code)]` at the top of `main.rs`. All three direct
  dependencies are pure Rust (crossterm's platform bindings via `libc`/Windows
  APIs are the only FFI in the transitive tree, and are unavoidable for
  raw-mode termios calls).
- **Escape-sequence injection** is the primary risk and is neutralized by the
  mandatory sanitization rule in Section 5. Add a unit test that feeds a
  document containing a raw ESC byte followed by an OSC 52 clipboard-write
  sequence and asserts that no laid-out `Span` contains byte `0x1b`.
- **Robustness on adversarial input:** a unit test must render the corpus file
  plus a 10 kB pseudo-random valid-UTF-8 string at widths 1, 2, and 40 without
  panicking. Generate the string deterministically with a hand-rolled LCG and a
  fixed seed — do NOT add a `rand` dependency (Section 2 forbids extra crates).
- Commit `Cargo.lock` to the repository. After the first successful build and
  after any `cargo update`, run `cargo audit` and fail on any advisory.
  (Checked against RustSec as of July 2026: no advisories for pulldown-cmark,
  crossterm, or unicode-width.)
- Do not add features that write anywhere (clipboard integration, export,
  history files) without an explicit new requirement.

## 13. Stretch goals (do not build unless asked)

- `--watch`: re-render on file change (via polling mtime every 500 ms — still no
  extra dependency), preserving scroll position by nearest heading.
- Reading from stdin (`mdv -`) with static output when piped.
- OSC 8 clickable hyperlinks behind a `--hyperlinks` flag; if implemented, emit
  OSC 8 only for URLs whose scheme is exactly `http` or `https` (never `file`,
  `javascript`, or anything else), and percent-escape the URL before emission.
