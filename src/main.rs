#![forbid(unsafe_code)]

mod input;
mod layout;
mod render;
mod style;
mod view;

use std::env;
use std::fs;
use std::io;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{read, Event, KeyEventKind};
use crossterm::style::{Attribute, Print, SetAttribute, SetForegroundColor};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

const USAGE: &str = "Usage: mdv <FILE>\n       mdv --help | -h\n       mdv --version | -V";

const KEYBINDINGS: &str = "\
Keybindings (Normal mode):
  j, Down                     scroll down 1 line
  k, Up                       scroll up 1 line
  d, PageDown, Space          scroll down half a screen
  u, PageUp                   scroll up half a screen
  g, Home                     go to top
  G, End                      go to bottom
  t                           toggle TOC overlay
  /                           start search input
  n / N                       next / previous search match
  Esc                         close overlay / cancel search input / clear search highlights
  q, Ctrl-C                   quit";

#[derive(Debug)]
struct RunConfig {
    contents: String,
    /// Basename of the file being viewed, for the Section 8 status bar.
    filename: String,
}

#[derive(Debug)]
enum Cli {
    Run(RunConfig),
    Help,
    Version,
}

/// Strips the platform-specific `" (os error N)"` suffix `io::Error`'s
/// `Display` impl appends, so stderr output matches the one-line format
/// in `docs/mdv-build-plan.md` Section 3 across platforms.
fn clean_io_message(e: &io::Error) -> String {
    let full = e.to_string();
    match full.find(" (os error") {
        Some(idx) => full[..idx].to_string(),
        None => full,
    }
}

/// Converts raw argv (`OsString`, which on Unix is arbitrary bytes and is
/// *not* guaranteed to be valid UTF-8) into `String`s before `parse_args` ever
/// sees them. `env::args()` panics on a non-UTF-8 argument; a hostile or
/// merely mis-encoded filename must not crash the process, so this collects
/// via `args_os()` and turns a conversion failure into the same one-line
/// stderr contract every other invalid-argument case uses (Section 3).
fn collect_args<I: IntoIterator<Item = std::ffi::OsString>>(
    args: I,
) -> Result<Vec<String>, String> {
    args.into_iter()
        .map(|arg| {
            arg.into_string()
                .map_err(|_| "mdv: argument is not valid UTF-8".to_string())
        })
        .collect()
}

fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Cli, String> {
    let mut path: Option<String> = None;
    for arg in args {
        match arg.as_str() {
            "--help" | "-h" => return Ok(Cli::Help),
            "--version" | "-V" => return Ok(Cli::Version),
            _ if arg.starts_with('-') => {
                return Err(format!("mdv: unknown option '{arg}'"));
            }
            _ => {
                if path.is_some() {
                    return Err(format!("mdv: unexpected argument '{arg}'"));
                }
                path = Some(arg);
            }
        }
    }

    let path = path.ok_or_else(|| "mdv: missing required argument <FILE>".to_string())?;
    let path = PathBuf::from(path);

    let bytes = fs::read(&path).map_err(|e| {
        format!(
            "mdv: cannot read '{}': {}",
            path.display(),
            clean_io_message(&e)
        )
    })?;

    let contents = String::from_utf8(bytes)
        .map_err(|_| format!("mdv: '{}' is not valid UTF-8", path.display()))?;

    let filename = path
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(path.to_str().unwrap_or(""))
        .to_string();

    Ok(Cli::Run(RunConfig { contents, filename }))
}

fn ensure_stdout_is_tty() -> Result<(), String> {
    if io::stdout().is_terminal() {
        Ok(())
    } else {
        Err("mdv: interactive viewer requires a terminal".to_string())
    }
}

/// Leaves the alternate screen, shows the cursor, and disables raw mode.
/// Idempotent and infallible from the caller's perspective (errors are
/// swallowed): called both from the panic hook, where little can be done
/// about a further failure, and from `TerminalGuard`'s `Drop`.
fn restore_terminal() {
    let _ = execute!(io::stdout(), Show, LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

/// RAII guard for the terminal lifecycle (build plan Section 4): entering
/// installs a panic hook that restores the terminal *before* the panic
/// message prints, then chains to the previously installed hook. Dropping
/// the guard restores the terminal on every other exit path (normal return,
/// `?` propagation, `std::process::exit` is NOT covered — this codebase
/// never calls it).
struct TerminalGuard;

impl TerminalGuard {
    /// Steps run in the order the build plan specifies (alternate screen,
    /// raw mode, hide cursor). If a step after the first one fails, the
    /// terminal is left in a partially set-up state — no `TerminalGuard`
    /// exists yet to restore it via `Drop`, and the panic hook isn't
    /// installed yet either — so each such failure explicitly restores
    /// before returning the error.
    fn enter() -> io::Result<Self> {
        execute!(io::stdout(), EnterAlternateScreen)?;

        if let Err(e) = enable_raw_mode() {
            restore_terminal();
            return Err(e);
        }

        if let Err(e) = execute!(io::stdout(), Hide) {
            restore_terminal();
            return Err(e);
        }

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            restore_terminal();
            previous_hook(info);
        }));

        Ok(TerminalGuard)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        restore_terminal();
    }
}

/// `offset * 100 / max(1, max_offset)`, clamped to 0-100 (Section 8's exact
/// formula; `max_offset()` already equals `lines.len().saturating_sub
/// (viewport_height)`). Always numeric — `offset`/`max_offset` can diverge
/// from the `<= max_offset` invariant other scrolling maintains (e.g. after
/// `toc_jump`, which the Phase 5 plan pins to the top row even at the very
/// end of the document), so this is explicitly clamped rather than assumed
/// to land in range.
fn scroll_percentage(view: &view::ViewState) -> usize {
    let denominator = view.max_offset().max(1);
    (view.offset() * 100 / denominator).min(100)
}

/// Picks the first candidate (in order: most detailed to least) whose
/// display width fits alongside `filename_width` within `width`, requiring a
/// 1-column gutter when the candidate is non-empty. Falls back to the last
/// candidate (conventionally `""`) if none fit — the caller never truncates
/// the filename itself, so an overlong filename is simply allowed to run
/// past `width`, matching every other narrow-terminal path in this codebase
/// (clip/omit content, never panic).
fn pick_fitting_right_text(filename_width: usize, candidates: &[&str], width: usize) -> String {
    for &candidate in candidates {
        let candidate_width = UnicodeWidthStr::width(candidate);
        let gutter = if candidate.is_empty() { 0 } else { 1 };
        if filename_width + gutter + candidate_width <= width {
            return candidate.to_string();
        }
    }
    candidates.last().copied().unwrap_or("").to_string()
}

/// Builds the exact Section 8 status-bar row (or its `status_message`
/// override), left-aligning `filename` and right-aligning the
/// percentage/count/hint block (or message), padded/truncated to exactly
/// `width` display columns. Truncation priority when it doesn't all fit:
/// drop the hint first, then the percentage/count block, before ever
/// truncating the filename (Phase 5 plan, Resolved design decisions).
fn status_bar_line(view: &view::ViewState, filename: &str, width: usize) -> String {
    let filename_width = UnicodeWidthStr::width(filename);

    let right = if let Some(message) = view.status_message() {
        pick_fitting_right_text(filename_width, &[message, ""], width)
    } else {
        let percentage = scroll_percentage(view);
        let current = view.offset() + 1;
        let total = view.total_lines();
        let full = format!("{percentage}% · {current}/{total} · t:toc /:search q:quit");
        let without_hint = format!("{percentage}% · {current}/{total}");
        pick_fitting_right_text(filename_width, &[&full, &without_hint, ""], width)
    };

    let right_width = UnicodeWidthStr::width(right.as_str());
    let gutter_width = width.saturating_sub(filename_width + right_width);
    format!("{filename}{}{right}", " ".repeat(gutter_width))
}

/// Builds the `Mode::SearchInput` input line: `/query▌`, left-truncating
/// the *displayed* query (not the stored buffer) when `/` + query + the
/// cursor marker don't fit `width`, so the cursor marker stays visible —
/// truncating from the left keeps the end of the query (nearest the
/// cursor) rather than its start.
fn search_input_line(query: &str, width: usize) -> String {
    const CURSOR: &str = "▌";
    let prefix_width = 1; // "/"
    let cursor_width = UnicodeWidthStr::width(CURSOR);
    let budget = width.saturating_sub(prefix_width + cursor_width);

    let mut chars: Vec<char> = query.chars().collect();
    let mut query_width = UnicodeWidthStr::width(query);
    while query_width > budget && !chars.is_empty() {
        let removed = chars.remove(0);
        query_width -= UnicodeWidthChar::width(removed).unwrap_or(0);
    }
    let display_query: String = chars.into_iter().collect();

    format!("/{display_query}{CURSOR}")
}

/// Truncates `text` to at most `max_width` display columns, appending a
/// trailing `…` (reserving 1 column for it) when it doesn't fit as-is.
/// Mirrors `layout::truncate_verbatim`'s reserve-a-column approach, but for
/// plain strings rather than styled spans — this module's TOC text has no
/// per-run styling to preserve.
fn truncate_with_ellipsis(text: &str, max_width: usize) -> String {
    if UnicodeWidthStr::width(text) <= max_width {
        return text.to_string();
    }
    if max_width == 0 {
        return String::new();
    }

    let budget = max_width - 1;
    let mut result = String::new();
    let mut used = 0;
    for c in text.chars() {
        let w = UnicodeWidthChar::width(c).unwrap_or(0);
        if used + w > budget {
            break;
        }
        result.push(c);
        used += w;
    }
    result.push('…');
    result
}

/// Section 6's TOC overlay geometry: `(box_width, box_height, box_left,
/// box_top)`. Width and height are clamped to a minimum of 1 rather than
/// underflowing or drawing a zero-size box at very small terminal sizes;
/// left/top center the box within the terminal.
fn toc_box_geometry(
    heading_count: usize,
    term_width: u16,
    term_height: u16,
) -> (usize, usize, usize, usize) {
    let box_width = 60.min((term_width as usize).saturating_sub(4)).max(1);
    let box_height = (heading_count + 2)
        .min((term_height as usize).saturating_sub(4))
        .max(1);
    let box_left = (term_width as usize).saturating_sub(box_width) / 2;
    let box_top = (term_height as usize).saturating_sub(box_height) / 2;
    (box_width, box_height, box_left, box_top)
}

/// Renders the Section 6 TOC overlay as an additional pass on top of
/// whatever `draw` already put on screen: a centered box listing every
/// heading, 2-space indented per level below H1, the current selection in
/// reverse, overflowing heading text truncated with a trailing `…`, and
/// `view.toc_scroll()`/`view.toc_cursor()` picking the visible window when
/// there are more headings than rows. No-op when there are no headings —
/// `ViewState::open_toc` already guards against entering `Mode::Toc` in that
/// case, so this is purely defensive.
fn draw_toc_overlay(
    headings: &[render::TocEntry],
    view: &view::ViewState,
    term_width: u16,
    term_height: u16,
) -> io::Result<()> {
    if headings.is_empty() {
        return Ok(());
    }

    let (box_width, box_height, box_left, box_top) =
        toc_box_geometry(headings.len(), term_width, term_height);
    let interior_width = box_width.saturating_sub(2);
    let visible_rows = box_height.saturating_sub(2);

    let mut stdout = io::stdout();

    let title = truncate_with_ellipsis(" Contents ", interior_width);
    let title_width = UnicodeWidthStr::width(title.as_str());
    let top_fill = "─".repeat(interior_width.saturating_sub(title_width));
    queue!(stdout, MoveTo(box_left as u16, box_top as u16))?;
    queue!(stdout, Print(format!("┌{title}{top_fill}┐")))?;

    let scroll = view.toc_scroll();
    let cursor = view.toc_cursor();
    for row in 0..visible_rows {
        queue!(stdout, MoveTo(box_left as u16, (box_top + 1 + row) as u16))?;
        queue!(stdout, Print("│"))?;

        let index = scroll + row;
        let display = match headings.get(index) {
            Some(entry) => {
                let indent = "  ".repeat(entry.level.saturating_sub(1) as usize);
                truncate_with_ellipsis(&format!("{indent}{}", entry.text), interior_width)
            }
            None => String::new(),
        };
        let pad =
            " ".repeat(interior_width.saturating_sub(UnicodeWidthStr::width(display.as_str())));

        if index == cursor {
            queue!(stdout, SetAttribute(Attribute::Reverse))?;
            queue!(stdout, Print(format!("{display}{pad}")))?;
            queue!(stdout, SetAttribute(Attribute::Reset))?;
        } else {
            queue!(stdout, Print(format!("{display}{pad}")))?;
        }

        queue!(stdout, Print("│"))?;
    }

    queue!(
        stdout,
        MoveTo(box_left as u16, (box_top + box_height - 1) as u16)
    )?;
    queue!(stdout, Print(format!("└{}┘", "─".repeat(interior_width))))?;

    stdout.flush()
}

/// Clears the screen and prints the current viewport's visible line slice,
/// followed by the Section 8 status bar as the last terminal row. The only
/// place output is written to the terminal (build plan Section 4). `\r\n`
/// (not `\n`) is required between lines: raw mode disables the
/// newline-to-CRLF translation a cooked terminal normally performs, so a
/// bare `\n` would move down without returning to column 0. The separator is
/// placed *between* lines, never after the last one: printing it after the
/// last line pushes the cursor past the bottom row when the viewport is
/// completely full, which triggers a terminal scroll and shifts the just-drawn
/// content up by one line.
///
/// Each span's full style is set explicitly (not diffed against the
/// previous span) and reset immediately after its text, so style never
/// bleeds onto whatever prints next regardless of what preceded it.
fn draw(view: &view::ViewState, filename: &str, width: u16, height: u16) -> io::Result<()> {
    let mut stdout = io::stdout();
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    for (i, line) in view.visible_lines().iter().enumerate() {
        if i > 0 {
            queue!(stdout, Print("\r\n"))?;
        }

        let highlighted;
        let spans: &[style::Span] = if let Some(search) = view.search() {
            highlighted = view::highlight_matches(line, &search.query);
            &highlighted.spans
        } else {
            &line.spans
        };

        for span in spans {
            if let Some(color) = span.style.fg {
                queue!(stdout, SetForegroundColor(color))?;
            }
            if span.style.bold {
                queue!(stdout, SetAttribute(Attribute::Bold))?;
            }
            if span.style.dim {
                queue!(stdout, SetAttribute(Attribute::Dim))?;
            }
            if span.style.italic {
                queue!(stdout, SetAttribute(Attribute::Italic))?;
            }
            if span.style.underline {
                queue!(stdout, SetAttribute(Attribute::Underlined))?;
            }
            if span.style.strikethrough {
                queue!(stdout, SetAttribute(Attribute::CrossedOut))?;
            }
            if span.style.reverse {
                queue!(stdout, SetAttribute(Attribute::Reverse))?;
            }
            queue!(stdout, Print(&span.text))?;
            queue!(stdout, SetAttribute(Attribute::Reset))?;
        }
    }

    let status_row = height.saturating_sub(1);
    queue!(stdout, MoveTo(0, status_row))?;
    if view.mode() == view::Mode::SearchInput {
        let line = search_input_line(view.search_input(), width as usize);
        queue!(stdout, Print(&line))?;
    } else {
        let line = status_bar_line(view, filename, width as usize);
        queue!(stdout, SetAttribute(Attribute::Reverse))?;
        queue!(stdout, Print(&line))?;
        queue!(stdout, SetAttribute(Attribute::Reset))?;
    }

    stdout.flush()
}

/// Draws a full frame: content + status bar, plus the TOC overlay on top
/// when `view.mode() == Mode::Toc`. The single entry point every redraw in
/// `run` goes through, so the overlay never gets forgotten at a call site.
fn render_frame(
    view: &view::ViewState,
    filename: &str,
    headings: &[render::TocEntry],
    width: u16,
    height: u16,
) -> io::Result<()> {
    draw(view, filename, width, height)?;
    if view.mode() == view::Mode::Toc {
        draw_toc_overlay(headings, view, width, height)?;
    }
    Ok(())
}

fn run(config: RunConfig) -> Result<ExitCode, String> {
    ensure_stdout_is_tty()?;
    let _terminal = TerminalGuard::enter().map_err(|e| {
        format!(
            "mdv: failed to initialize terminal: {}",
            clean_io_message(&e)
        )
    })?;

    let document = render::build_document(&config.contents);

    let (mut term_width, mut term_height) = size().map_err(|e| {
        format!(
            "mdv: failed to query terminal size: {}",
            clean_io_message(&e)
        )
    })?;
    let layout_result = layout::wrap(&document, term_width as usize);
    let mut view = view::ViewState::new(
        layout_result.lines,
        layout_result.heading_lines,
        (term_height as usize).saturating_sub(1),
    );

    let io_err = |e: io::Error| format!("mdv: failed to draw: {}", clean_io_message(&e));
    render_frame(
        &view,
        &config.filename,
        &document.headings,
        term_width,
        term_height,
    )
    .map_err(io_err)?;

    while let Ok(event) = read() {
        match event {
            Event::Resize(width, height) => {
                term_width = width;
                term_height = height;
                let layout_result = layout::wrap(&document, term_width as usize);
                view.set_layout(
                    layout_result.lines,
                    layout_result.heading_lines,
                    (term_height as usize).saturating_sub(1),
                );
                render_frame(
                    &view,
                    &config.filename,
                    &document.headings,
                    term_width,
                    term_height,
                )
                .map_err(io_err)?;
            }
            Event::Key(key_event) => {
                if key_event.kind != KeyEventKind::Press {
                    continue;
                }

                // Cleared unconditionally on every processed keypress,
                // whether or not it maps to a recognized Action (Phase 5
                // plan, Resolved design decisions).
                let had_status_message = view.status_message().is_some();
                view.clear_status_message();

                let action = match input::map(key_event, view.mode()) {
                    Some(action) => action,
                    None => {
                        if had_status_message {
                            render_frame(
                                &view,
                                &config.filename,
                                &document.headings,
                                term_width,
                                term_height,
                            )
                            .map_err(io_err)?;
                        }
                        continue;
                    }
                };

                if action == input::Action::Quit {
                    break;
                }

                let previous_offset = view.offset();
                match action {
                    input::Action::LineDown => view.scroll_down(1),
                    input::Action::LineUp => view.scroll_up(1),
                    input::Action::HalfPageDown => view.half_page_down(),
                    input::Action::HalfPageUp => view.half_page_up(),
                    input::Action::Top => view.jump_to_top(),
                    input::Action::Bottom => view.jump_to_bottom(),
                    input::Action::Quit => {
                        unreachable!("Quit is handled above before this match")
                    }
                    // Wired up in task 10.
                    _ => {}
                }

                if view.offset() != previous_offset || had_status_message {
                    render_frame(
                        &view,
                        &config.filename,
                        &document.headings,
                        term_width,
                        term_height,
                    )
                    .map_err(io_err)?;
                }
            }
            _ => {}
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn main() -> ExitCode {
    let args = match collect_args(env::args_os().skip(1)) {
        Ok(args) => args,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    match parse_args(args) {
        Ok(Cli::Help) => {
            println!("{USAGE}\n\n{KEYBINDINGS}");
            ExitCode::SUCCESS
        }
        Ok(Cli::Version) => {
            println!("mdv {}", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Ok(Cli::Run(config)) => match run(config) {
            Ok(code) => code,
            Err(message) => {
                eprintln!("{message}");
                ExitCode::FAILURE
            }
        },
        Err(message) => {
            eprintln!("{message}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    fn args(strs: &[&str]) -> Vec<String> {
        strs.iter().map(|s| s.to_string()).collect()
    }

    struct TempFile {
        path: PathBuf,
    }

    impl TempFile {
        fn new(bytes: &[u8]) -> Self {
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            let n = COUNTER.fetch_add(1, Ordering::Relaxed);
            let path =
                env::temp_dir().join(format!("mdv-test-{}-{}-{n}.md", std::process::id(), n));
            fs::write(&path, bytes).expect("write temp file");
            TempFile { path }
        }
    }

    impl Drop for TempFile {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.path);
        }
    }

    #[test]
    #[cfg(unix)]
    fn non_utf8_argument_is_an_error_not_a_panic() {
        use std::ffi::OsString;
        use std::os::unix::ffi::OsStringExt;

        let bad_arg = OsString::from_vec(vec![0xff, 0xfe]);
        let err = collect_args(vec![bad_arg]).expect_err("expected error");
        assert_eq!(err, "mdv: argument is not valid UTF-8");
    }

    #[test]
    fn no_arguments_is_an_error() {
        let err = parse_args(args(&[])).expect_err("expected error");
        assert_eq!(err, "mdv: missing required argument <FILE>");
    }

    #[test]
    fn unknown_flag_is_an_error() {
        let err = parse_args(args(&["--bogus"])).expect_err("expected error");
        assert_eq!(err, "mdv: unknown option '--bogus'");
    }

    #[test]
    fn two_positional_arguments_is_an_error() {
        let err = parse_args(args(&["a.md", "b.md"])).expect_err("expected error");
        assert_eq!(err, "mdv: unexpected argument 'b.md'");
    }

    #[test]
    fn help_long_flag() {
        assert!(matches!(parse_args(args(&["--help"])), Ok(Cli::Help)));
    }

    #[test]
    fn help_short_flag() {
        assert!(matches!(parse_args(args(&["-h"])), Ok(Cli::Help)));
    }

    #[test]
    fn version_long_flag() {
        assert!(matches!(parse_args(args(&["--version"])), Ok(Cli::Version)));
    }

    #[test]
    fn version_short_flag() {
        assert!(matches!(parse_args(args(&["-V"])), Ok(Cli::Version)));
    }

    #[test]
    fn flags_take_precedence_over_a_preceding_positional_argument() {
        assert!(matches!(
            parse_args(args(&["notes.md", "--help"])),
            Ok(Cli::Help)
        ));
    }

    #[test]
    fn unreadable_file_is_an_error() {
        let path = env::temp_dir().join("mdv-test-does-not-exist.md");
        let _ = fs::remove_file(&path);
        let err = parse_args(args(&[path.to_str().unwrap()])).expect_err("expected error");
        assert_eq!(
            err,
            format!(
                "mdv: cannot read '{}': No such file or directory",
                path.display()
            )
        );
    }

    #[test]
    fn non_utf8_file_is_an_error() {
        let file = TempFile::new(&[0xff, 0xfe, 0x00, 0xff]);
        let err = parse_args(args(&[file.path.to_str().unwrap()])).expect_err("expected error");
        assert_eq!(
            err,
            format!("mdv: '{}' is not valid UTF-8", file.path.display())
        );
    }

    #[test]
    fn valid_file_parses_to_run() {
        let file = TempFile::new(b"# Hello\n\nworld\n");
        match parse_args(args(&[file.path.to_str().unwrap()])).expect("expected Ok") {
            Cli::Run(config) => {
                assert_eq!(config.contents, "# Hello\n\nworld\n");
                assert_eq!(
                    config.filename,
                    file.path.file_name().unwrap().to_str().unwrap()
                );
            }
            _ => panic!("expected Cli::Run"),
        }
    }

    /// Re-serializes one `Span` back to a plain-text marker form, applying
    /// markers outermost-to-innermost per the fixed scheme (plan task 8):
    /// bold `**`, italic `*`, strikethrough `~~`, `style::CODE` foreground
    /// `` ` ``. The corpus never combines inline code with bold/italic/
    /// strikethrough on the same run, so these four never need to interleave
    /// beyond simple nesting. Dim/underline/link-color/heading-color have no
    /// natural plain-text marker and are intentionally left unmarked — they
    /// are covered by the `render.rs`/`layout.rs` unit tests instead.
    fn serialize_span(span: &style::Span) -> String {
        let mut text = span.text.clone();
        if span.style.fg == Some(style::CODE) {
            text = format!("`{text}`");
        }
        if span.style.strikethrough {
            text = format!("~~{text}~~");
        }
        if span.style.italic {
            text = format!("*{text}*");
        }
        if span.style.bold {
            text = format!("**{text}**");
        }
        text
    }

    fn serialize_lines(lines: &[layout::Line]) -> String {
        lines
            .iter()
            .map(|line| line.spans.iter().map(serialize_span).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn corpus_snapshot_at_width_80() {
        let corpus_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus.md");
        let snapshot_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/snapshots/corpus.txt");

        let markdown = fs::read_to_string(corpus_path).expect("read tests/corpus.md");
        let document = render::build_document(&markdown);
        let result = layout::wrap(&document, 80);
        let rendered = serialize_lines(&result.lines);

        if env::var("UPDATE_SNAPSHOTS").is_ok() {
            fs::write(snapshot_path, &rendered).expect("write snapshot");
            return;
        }

        let expected = fs::read_to_string(snapshot_path).expect("read tests/snapshots/corpus.txt");
        assert_eq!(
            rendered, expected,
            "corpus snapshot mismatch; run with UPDATE_SNAPSHOTS=1 to regenerate"
        );
    }

    /// Section 12's other robustness requirement (no panic at widths 1/2/40
    /// on adversarial input) extended to also cover the real corpus, now
    /// that it exists — `layout::tests::no_panic_on_adversarial_input_at_narrow_widths`
    /// covers the seeded-LCG half.
    #[test]
    fn corpus_renders_without_panicking_at_narrow_widths() {
        let corpus_path = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/corpus.md");
        let markdown = fs::read_to_string(corpus_path).expect("read tests/corpus.md");
        let document = render::build_document(&markdown);
        for width in [1, 2, 40] {
            let _ = layout::wrap(&document, width);
        }
    }

    fn view_with(total_lines: usize, viewport_height: usize) -> view::ViewState {
        let lines: Vec<layout::Line> = (0..total_lines)
            .map(|_| layout::Line { spans: Vec::new() })
            .collect();
        view::ViewState::new(lines, Vec::new(), viewport_height)
    }

    #[test]
    fn scroll_percentage_is_0_at_top_and_100_at_bottom() {
        let mut view = view_with(100, 10);
        assert_eq!(scroll_percentage(&view), 0);
        view.jump_to_bottom();
        assert_eq!(scroll_percentage(&view), 100);
    }

    #[test]
    fn scroll_percentage_never_exceeds_100_even_past_max_offset() {
        // A document that fits entirely in the viewport has max_offset() ==
        // 0; nothing should divide by zero or exceed 100%.
        let view = view_with(5, 10);
        assert_eq!(scroll_percentage(&view), 0);
    }

    #[test]
    fn pick_fitting_right_text_prefers_the_first_candidate_that_fits() {
        let candidates = ["full hint here", "short", ""];
        assert_eq!(
            pick_fitting_right_text(10, &candidates, 30),
            "full hint here"
        );
        assert_eq!(pick_fitting_right_text(10, &candidates, 17), "short");
        assert_eq!(pick_fitting_right_text(10, &candidates, 11), "");
    }

    #[test]
    fn pick_fitting_right_text_falls_back_to_empty_when_filename_alone_overflows() {
        let candidates = ["full hint here", "short", ""];
        assert_eq!(pick_fitting_right_text(50, &candidates, 10), "");
    }

    #[test]
    fn status_bar_line_lays_out_filename_left_and_stats_right() {
        let view = view_with(284, 10);
        let line = status_bar_line(&view, "notes.md", 80);
        assert!(line.starts_with("notes.md"));
        assert!(line.ends_with("t:toc /:search q:quit"));
        assert!(line.contains("0% · 1/284"));
        assert_eq!(UnicodeWidthStr::width(line.as_str()), 80);
    }

    #[test]
    fn status_bar_line_drops_hint_then_stats_before_ever_truncating_filename() {
        let view = view_with(284, 10);
        let filename = "notes.md";
        let filename_width = UnicodeWidthStr::width(filename);

        let full = "0% · 1/284 · t:toc /:search q:quit";
        let without_hint = "0% · 1/284";
        let full_needed = filename_width + 1 + UnicodeWidthStr::width(full);
        let without_hint_needed = filename_width + 1 + UnicodeWidthStr::width(without_hint);

        let with_hint = status_bar_line(&view, filename, full_needed);
        assert!(with_hint.contains("t:toc"));

        let dropped_hint = status_bar_line(&view, filename, full_needed - 1);
        assert!(
            !dropped_hint.contains("t:toc"),
            "hint must be dropped first"
        );
        assert!(dropped_hint.contains("0% · 1/284"));

        let dropped_stats = status_bar_line(&view, filename, without_hint_needed - 1);
        assert!(!dropped_stats.contains('%'), "stats must be dropped next");
        assert!(
            dropped_stats.starts_with(filename),
            "filename itself must never be truncated"
        );
    }

    #[test]
    fn status_bar_line_shows_status_message_instead_of_stats() {
        let mut view = view_with(284, 10);
        view.set_status_message("Pattern not found: xyz".to_string());
        let line = status_bar_line(&view, "notes.md", 80);
        assert!(line.contains("Pattern not found: xyz"));
        assert!(!line.contains('%'));
    }

    #[test]
    fn status_bar_line_never_panics_at_degenerate_widths() {
        let view = view_with(5, 10);
        for width in [0, 1, 2] {
            let _ = status_bar_line(&view, "a-very-long-filename.md", width);
        }
    }

    #[test]
    fn truncate_with_ellipsis_leaves_short_text_unchanged() {
        assert_eq!(truncate_with_ellipsis("hello", 10), "hello");
        assert_eq!(truncate_with_ellipsis("hello", 5), "hello");
    }

    #[test]
    fn truncate_with_ellipsis_reserves_a_column_for_the_mark() {
        assert_eq!(truncate_with_ellipsis("hello world", 5), "hell…");
    }

    #[test]
    fn truncate_with_ellipsis_never_panics_at_zero_width() {
        assert_eq!(truncate_with_ellipsis("hello", 0), "");
    }

    #[test]
    fn toc_box_geometry_caps_width_at_60_and_clamps_to_terminal_size() {
        let (width, height, left, top) = toc_box_geometry(3, 200, 100);
        assert_eq!(width, 60);
        assert_eq!(height, 5); // 3 headings + 2 border rows
        assert_eq!(left, (200 - 60) / 2);
        assert_eq!(top, (100 - 5) / 2);
    }

    #[test]
    fn toc_box_geometry_shrinks_to_fit_a_small_terminal_without_underflow() {
        let (width, height, _, _) = toc_box_geometry(50, 10, 8);
        assert_eq!(width, 6); // 10 - 4
        assert_eq!(height, 4); // 8 - 4
    }

    #[test]
    fn toc_box_geometry_never_underflows_at_degenerate_terminal_sizes() {
        for (w, h) in [(0, 0), (1, 1), (3, 3), (4, 4)] {
            let (width, height, left, top) = toc_box_geometry(5, w, h);
            assert!(width >= 1);
            assert!(height >= 1);
            assert!(left <= w as usize);
            assert!(top <= h as usize);
        }
    }

    #[test]
    fn draw_toc_overlay_never_panics_at_degenerate_terminal_sizes() {
        let headings = vec![
            render::TocEntry {
                level: 1,
                text: "Intro".to_string(),
                block_index: 0,
            },
            render::TocEntry {
                level: 2,
                text: "Details".to_string(),
                block_index: 1,
            },
        ];
        let view = view_with(20, 10);
        for (w, h) in [(0, 0), (1, 1), (3, 3), (5, 5), (80, 24)] {
            let _ = draw_toc_overlay(&headings, &view, w, h);
        }
    }

    #[test]
    fn search_input_line_shows_the_query_and_cursor_when_it_fits() {
        assert_eq!(search_input_line("hello", 20), "/hello▌");
    }

    #[test]
    fn search_input_line_truncates_the_query_from_the_left_to_keep_the_cursor_visible() {
        // width 5 => budget for the query is 5 - 1 ("/") - 1 (cursor) = 3,
        // so only the last 3 chars of the query survive.
        assert_eq!(search_input_line("hello world", 5), "/rld▌");
    }

    #[test]
    fn search_input_line_never_panics_at_degenerate_widths() {
        for width in [0, 1, 2] {
            let _ = search_input_line("hello", width);
        }
    }
}
