use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::view::Mode;

/// All interaction-model actions across every `Mode` (Phase 1's scrolling
/// subset plus Phase 5's search/TOC additions). `SearchChar`/`SearchBackspace`/
/// `SearchExecute` are only ever produced in `Mode::SearchInput`; `TocUp`/
/// `TocDown`/`TocJump` only in `Mode::Toc` — kept distinct from `LineUp`/
/// `LineDown`/etc. because those carry `ViewState` viewport-scrolling
/// semantics while the Toc variants carry `toc_cursor` semantics, and
/// conflating them would just push the mode disambiguation onto the caller.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    LineUp,
    LineDown,
    HalfPageUp,
    HalfPageDown,
    Top,
    Bottom,
    Quit,
    ToggleToc,
    StartSearch,
    NextMatch,
    PrevMatch,
    Escape,
    SearchChar(char),
    SearchBackspace,
    SearchExecute,
    TocUp,
    TocDown,
    TocJump,
}

/// Maps a key event to an `Action`, dispatching on the current `Mode` since
/// `SearchInput` and `Toc` override Normal-mode keybindings (Section 6:
/// "SearchInput and Toc modes override keys"). Returns `None` for any key not
/// bound for the current mode — the caller should treat that as a no-op.
/// Only key-press events are mapped; release/repeat events (emitted on some
/// platforms) are ignored. Ctrl-C quits immediately regardless of mode, as a
/// safety escape hatch.
pub fn map(event: KeyEvent, mode: Mode) -> Option<Action> {
    if event.kind != KeyEventKind::Press {
        return None;
    }

    if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }

    match mode {
        Mode::Normal => map_normal(event.code),
        Mode::SearchInput => map_search_input(event.code),
        Mode::Toc => map_toc(event.code),
    }
}

/// Section 7's exact, complete Normal-mode keybinding table.
fn map_normal(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::LineDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::LineUp),
        KeyCode::Char('d') | KeyCode::PageDown | KeyCode::Char(' ') => Some(Action::HalfPageDown),
        KeyCode::Char('u') | KeyCode::PageUp => Some(Action::HalfPageUp),
        KeyCode::Char('g') | KeyCode::Home => Some(Action::Top),
        KeyCode::Char('G') | KeyCode::End => Some(Action::Bottom),
        KeyCode::Char('t') => Some(Action::ToggleToc),
        KeyCode::Char('/') => Some(Action::StartSearch),
        KeyCode::Char('n') => Some(Action::NextMatch),
        KeyCode::Char('N') => Some(Action::PrevMatch),
        KeyCode::Esc => Some(Action::Escape),
        KeyCode::Char('q') => Some(Action::Quit),
        _ => None,
    }
}

/// `/query▌` input line: printable chars append (including ones that are
/// otherwise Normal-mode bindings, e.g. `t`/`q`/`n` — they're literal here),
/// Backspace deletes, Enter executes, Esc cancels back to Normal.
fn map_search_input(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char(c) => Some(Action::SearchChar(c)),
        KeyCode::Backspace => Some(Action::SearchBackspace),
        KeyCode::Enter => Some(Action::SearchExecute),
        KeyCode::Esc => Some(Action::Escape),
        _ => None,
    }
}

/// TOC overlay: `j`/`k`/arrows move, Enter jumps, Esc/`t`/`q` close the
/// overlay (`q` does not quit while the TOC is open — an explicit override
/// of the literal Section 7 table, see the Phase 5 plan's Resolved design
/// decisions).
fn map_toc(code: KeyCode) -> Option<Action> {
    match code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::TocDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::TocUp),
        KeyCode::Enter => Some(Action::TocJump),
        KeyCode::Esc | KeyCode::Char('t') | KeyCode::Char('q') => Some(Action::Escape),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    #[test]
    fn line_down_keys() {
        assert_eq!(
            map(press(KeyCode::Char('j')), Mode::Normal),
            Some(Action::LineDown)
        );
        assert_eq!(
            map(press(KeyCode::Down), Mode::Normal),
            Some(Action::LineDown)
        );
    }

    #[test]
    fn line_up_keys() {
        assert_eq!(
            map(press(KeyCode::Char('k')), Mode::Normal),
            Some(Action::LineUp)
        );
        assert_eq!(map(press(KeyCode::Up), Mode::Normal), Some(Action::LineUp));
    }

    #[test]
    fn half_page_down_keys() {
        assert_eq!(
            map(press(KeyCode::Char('d')), Mode::Normal),
            Some(Action::HalfPageDown)
        );
        assert_eq!(
            map(press(KeyCode::PageDown), Mode::Normal),
            Some(Action::HalfPageDown)
        );
        assert_eq!(
            map(press(KeyCode::Char(' ')), Mode::Normal),
            Some(Action::HalfPageDown)
        );
    }

    #[test]
    fn half_page_up_keys() {
        assert_eq!(
            map(press(KeyCode::Char('u')), Mode::Normal),
            Some(Action::HalfPageUp)
        );
        assert_eq!(
            map(press(KeyCode::PageUp), Mode::Normal),
            Some(Action::HalfPageUp)
        );
    }

    #[test]
    fn top_keys() {
        assert_eq!(
            map(press(KeyCode::Char('g')), Mode::Normal),
            Some(Action::Top)
        );
        assert_eq!(map(press(KeyCode::Home), Mode::Normal), Some(Action::Top));
    }

    #[test]
    fn bottom_keys() {
        assert_eq!(
            map(press(KeyCode::Char('G')), Mode::Normal),
            Some(Action::Bottom)
        );
        assert_eq!(map(press(KeyCode::End), Mode::Normal), Some(Action::Bottom));
    }

    #[test]
    fn quit_keys() {
        assert_eq!(
            map(press(KeyCode::Char('q')), Mode::Normal),
            Some(Action::Quit)
        );
        assert_eq!(
            map(
                KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
                Mode::Normal
            ),
            Some(Action::Quit)
        );
    }

    #[test]
    fn ctrl_c_quits_in_every_mode() {
        let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
        assert_eq!(map(ctrl_c, Mode::Normal), Some(Action::Quit));
        assert_eq!(map(ctrl_c, Mode::SearchInput), Some(Action::Quit));
        assert_eq!(map(ctrl_c, Mode::Toc), Some(Action::Quit));
    }

    #[test]
    fn normal_mode_toc_and_search_bindings() {
        assert_eq!(
            map(press(KeyCode::Char('t')), Mode::Normal),
            Some(Action::ToggleToc)
        );
        assert_eq!(
            map(press(KeyCode::Char('/')), Mode::Normal),
            Some(Action::StartSearch)
        );
        assert_eq!(
            map(press(KeyCode::Char('n')), Mode::Normal),
            Some(Action::NextMatch)
        );
        assert_eq!(
            map(press(KeyCode::Char('N')), Mode::Normal),
            Some(Action::PrevMatch)
        );
        assert_eq!(map(press(KeyCode::Esc), Mode::Normal), Some(Action::Escape));
    }

    #[test]
    fn unbound_keys_are_no_ops() {
        assert_eq!(map(press(KeyCode::Char('x')), Mode::Normal), None);
        assert_eq!(map(press(KeyCode::F(1)), Mode::Normal), None);
    }

    #[test]
    fn non_press_events_are_ignored() {
        let mut event = press(KeyCode::Char('q'));
        event.kind = KeyEventKind::Release;
        assert_eq!(map(event, Mode::Normal), None);
    }

    #[test]
    fn search_input_mode_treats_normal_mode_letters_as_literal_characters() {
        assert_eq!(
            map(press(KeyCode::Char('t')), Mode::SearchInput),
            Some(Action::SearchChar('t'))
        );
        assert_eq!(
            map(press(KeyCode::Char('q')), Mode::SearchInput),
            Some(Action::SearchChar('q'))
        );
        assert_eq!(
            map(press(KeyCode::Char('n')), Mode::SearchInput),
            Some(Action::SearchChar('n'))
        );
    }

    #[test]
    fn search_input_mode_editing_and_submission_keys() {
        assert_eq!(
            map(press(KeyCode::Backspace), Mode::SearchInput),
            Some(Action::SearchBackspace)
        );
        assert_eq!(
            map(press(KeyCode::Enter), Mode::SearchInput),
            Some(Action::SearchExecute)
        );
        assert_eq!(
            map(press(KeyCode::Esc), Mode::SearchInput),
            Some(Action::Escape)
        );
    }

    #[test]
    fn toc_mode_navigation_and_close_keys() {
        assert_eq!(
            map(press(KeyCode::Char('j')), Mode::Toc),
            Some(Action::TocDown)
        );
        assert_eq!(map(press(KeyCode::Down), Mode::Toc), Some(Action::TocDown));
        assert_eq!(
            map(press(KeyCode::Char('k')), Mode::Toc),
            Some(Action::TocUp)
        );
        assert_eq!(map(press(KeyCode::Up), Mode::Toc), Some(Action::TocUp));
        assert_eq!(map(press(KeyCode::Enter), Mode::Toc), Some(Action::TocJump));
        assert_eq!(map(press(KeyCode::Esc), Mode::Toc), Some(Action::Escape));
        assert_eq!(
            map(press(KeyCode::Char('t')), Mode::Toc),
            Some(Action::Escape)
        );
    }

    #[test]
    fn toc_mode_q_closes_the_overlay_rather_than_quitting() {
        assert_eq!(
            map(press(KeyCode::Char('q')), Mode::Toc),
            Some(Action::Escape)
        );
    }

    #[test]
    fn toc_mode_unbound_keys_are_no_ops() {
        assert_eq!(map(press(KeyCode::Char('x')), Mode::Toc), None);
        assert_eq!(map(press(KeyCode::PageDown), Mode::Toc), None);
    }
}
