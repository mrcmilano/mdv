#![forbid(unsafe_code)]

mod input;
mod view;

use std::env;
use std::fs;
use std::io;
use std::io::{IsTerminal, Write};
use std::path::PathBuf;
use std::process::ExitCode;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{read, Event};
use crossterm::style::Print;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, size, Clear, ClearType, EnterAlternateScreen,
    LeaveAlternateScreen,
};
use crossterm::{execute, queue};

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

struct RunConfig {
    contents: String,
}

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

    Ok(Cli::Run(RunConfig { contents }))
}

/// Security-critical sanitization (build plan Section 5), applied to the
/// whole file text before it is split into lines: strips `\r`, replaces tabs
/// with a single space, and replaces every other C0 control character, DEL,
/// and C1 control with U+FFFD. `\n` is preserved as the line separator.
/// Neutralizes escape-sequence injection — no other ANSI/OSC sequence in the
/// source file survives to reach the terminal.
fn sanitize(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\r' => {}
            '\n' => out.push('\n'),
            '\t' => out.push(' '),
            c if is_replaceable_control(c) => out.push('\u{FFFD}'),
            c => out.push(c),
        }
    }
    out
}

fn is_replaceable_control(c: char) -> bool {
    let code = c as u32;
    (0x00..=0x1F).contains(&code) || code == 0x7F || (0x80..=0x9F).contains(&code)
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

/// Clears the screen and prints the current viewport's visible line slice.
/// The only place output is written to the terminal (build plan Section 4).
/// `\r\n` (not `\n`) is required between lines: raw mode disables the
/// newline-to-CRLF translation a cooked terminal normally performs, so a
/// bare `\n` would move down without returning to column 0. The separator is
/// placed *between* lines, never after the last one: printing it after the
/// last line pushes the cursor past the bottom row when the viewport is
/// completely full, which triggers a terminal scroll and shifts the just-drawn
/// content up by one line.
fn draw(view: &view::ViewState) -> io::Result<()> {
    let mut stdout = io::stdout();
    queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    for (i, line) in view.visible_lines().iter().enumerate() {
        if i > 0 {
            queue!(stdout, Print("\r\n"))?;
        }
        queue!(stdout, Print(line))?;
    }
    stdout.flush()
}

fn run(config: RunConfig) -> Result<ExitCode, String> {
    ensure_stdout_is_tty()?;
    let _terminal = TerminalGuard::enter().map_err(|e| {
        format!(
            "mdv: failed to initialize terminal: {}",
            clean_io_message(&e)
        )
    })?;

    let sanitized = sanitize(&config.contents);
    let lines: Vec<String> = sanitized.lines().map(str::to_string).collect();

    let (_width, height) = size().map_err(|e| {
        format!(
            "mdv: failed to query terminal size: {}",
            clean_io_message(&e)
        )
    })?;
    let mut view = view::ViewState::new(lines, height as usize);

    let io_err = |e: io::Error| format!("mdv: failed to draw: {}", clean_io_message(&e));
    draw(&view).map_err(io_err)?;

    while let Ok(event) = read() {
        let Event::Key(key_event) = event else {
            continue;
        };

        let action = match input::map(key_event) {
            Some(action) => action,
            None => continue,
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
            input::Action::Quit => unreachable!("Quit is handled above before this match"),
        }

        if view.offset() != previous_offset {
            draw(&view).map_err(io_err)?;
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
        let err = collect_args(vec![bad_arg]).err().expect("expected error");
        assert_eq!(err, "mdv: argument is not valid UTF-8");
    }

    #[test]
    fn no_arguments_is_an_error() {
        let err = parse_args(args(&[])).err().expect("expected error");
        assert_eq!(err, "mdv: missing required argument <FILE>");
    }

    #[test]
    fn unknown_flag_is_an_error() {
        let err = parse_args(args(&["--bogus"]))
            .err()
            .expect("expected error");
        assert_eq!(err, "mdv: unknown option '--bogus'");
    }

    #[test]
    fn two_positional_arguments_is_an_error() {
        let err = parse_args(args(&["a.md", "b.md"]))
            .err()
            .expect("expected error");
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
        let err = parse_args(args(&[path.to_str().unwrap()]))
            .err()
            .expect("expected error");
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
        let err = parse_args(args(&[file.path.to_str().unwrap()]))
            .err()
            .expect("expected error");
        assert_eq!(
            err,
            format!("mdv: '{}' is not valid UTF-8", file.path.display())
        );
    }

    #[test]
    fn sanitize_strips_carriage_returns() {
        assert_eq!(sanitize("a\r\nb\rc"), "a\nbc");
    }

    #[test]
    fn sanitize_preserves_newlines_as_line_separators() {
        assert_eq!(sanitize("a\nb\nc"), "a\nb\nc");
    }

    #[test]
    fn sanitize_replaces_tabs_with_a_single_space() {
        assert_eq!(sanitize("a\tb\t\tc"), "a b  c");
    }

    #[test]
    fn sanitize_replaces_other_c0_controls_with_replacement_char() {
        assert_eq!(sanitize("a\u{0007}b\u{001b}c"), "a\u{FFFD}b\u{FFFD}c");
    }

    #[test]
    fn sanitize_replaces_del_and_c1_controls_with_replacement_char() {
        assert_eq!(
            sanitize("a\u{007F}b\u{0080}c\u{009F}d"),
            "a\u{FFFD}b\u{FFFD}c\u{FFFD}d"
        );
    }

    #[test]
    fn sanitize_neutralizes_esc_byte_followed_by_osc_52_sequence() {
        // A raw ESC byte followed by an OSC 52 clipboard-write sequence —
        // the primary escape-sequence-injection threat (Section 12).
        let malicious = "before\u{001b}]52;c;BASE64DATA==\u{0007}after";
        let sanitized = sanitize(malicious);
        assert!(
            !sanitized.contains('\u{001b}'),
            "sanitized text must not contain a raw ESC byte"
        );
        assert!(!sanitized.as_bytes().contains(&0x1b));
    }

    #[test]
    fn valid_file_parses_to_run() {
        let file = TempFile::new(b"# Hello\n\nworld\n");
        match parse_args(args(&[file.path.to_str().unwrap()])).expect("expected Ok") {
            Cli::Run(config) => {
                assert_eq!(config.contents, "# Hello\n\nworld\n");
            }
            _ => panic!("expected Cli::Run"),
        }
    }
}
