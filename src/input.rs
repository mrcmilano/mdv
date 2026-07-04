use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

/// Normal-mode scrolling/quit actions implemented in Phase 1. `t`, `/`, `n`,
/// `N` depend on TOC/search state that doesn't exist until Phase 5.
// Variants and `map` become live once task 7 wires the event loop to call it.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    LineUp,
    LineDown,
    HalfPageUp,
    HalfPageDown,
    Top,
    Bottom,
    Quit,
}

/// Maps a key event to an `Action`. Returns `None` for any key not bound in
/// Section 7's Normal-mode table (Phase 1 subset) — the caller should treat
/// that as a no-op. Only key-press events are mapped; release/repeat events
/// (emitted on some platforms) are ignored.
#[allow(dead_code)]
pub fn map(event: KeyEvent) -> Option<Action> {
    if event.kind != KeyEventKind::Press {
        return None;
    }

    if event.code == KeyCode::Char('c') && event.modifiers.contains(KeyModifiers::CONTROL) {
        return Some(Action::Quit);
    }

    match event.code {
        KeyCode::Char('j') | KeyCode::Down => Some(Action::LineDown),
        KeyCode::Char('k') | KeyCode::Up => Some(Action::LineUp),
        KeyCode::Char('d') | KeyCode::PageDown | KeyCode::Char(' ') => Some(Action::HalfPageDown),
        KeyCode::Char('u') | KeyCode::PageUp => Some(Action::HalfPageUp),
        KeyCode::Char('g') | KeyCode::Home => Some(Action::Top),
        KeyCode::Char('G') | KeyCode::End => Some(Action::Bottom),
        KeyCode::Char('q') => Some(Action::Quit),
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
        assert_eq!(map(press(KeyCode::Char('j'))), Some(Action::LineDown));
        assert_eq!(map(press(KeyCode::Down)), Some(Action::LineDown));
    }

    #[test]
    fn line_up_keys() {
        assert_eq!(map(press(KeyCode::Char('k'))), Some(Action::LineUp));
        assert_eq!(map(press(KeyCode::Up)), Some(Action::LineUp));
    }

    #[test]
    fn half_page_down_keys() {
        assert_eq!(map(press(KeyCode::Char('d'))), Some(Action::HalfPageDown));
        assert_eq!(map(press(KeyCode::PageDown)), Some(Action::HalfPageDown));
        assert_eq!(map(press(KeyCode::Char(' '))), Some(Action::HalfPageDown));
    }

    #[test]
    fn half_page_up_keys() {
        assert_eq!(map(press(KeyCode::Char('u'))), Some(Action::HalfPageUp));
        assert_eq!(map(press(KeyCode::PageUp)), Some(Action::HalfPageUp));
    }

    #[test]
    fn top_keys() {
        assert_eq!(map(press(KeyCode::Char('g'))), Some(Action::Top));
        assert_eq!(map(press(KeyCode::Home)), Some(Action::Top));
    }

    #[test]
    fn bottom_keys() {
        assert_eq!(map(press(KeyCode::Char('G'))), Some(Action::Bottom));
        assert_eq!(map(press(KeyCode::End)), Some(Action::Bottom));
    }

    #[test]
    fn quit_keys() {
        assert_eq!(map(press(KeyCode::Char('q'))), Some(Action::Quit));
        assert_eq!(
            map(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL)),
            Some(Action::Quit)
        );
    }

    #[test]
    fn unbound_keys_are_no_ops() {
        assert_eq!(map(press(KeyCode::Char('x'))), None);
        assert_eq!(map(press(KeyCode::Char('t'))), None);
        assert_eq!(map(press(KeyCode::Char('/'))), None);
        assert_eq!(map(press(KeyCode::Char('n'))), None);
        assert_eq!(map(press(KeyCode::Char('N'))), None);
        assert_eq!(map(press(KeyCode::Esc)), None);
        assert_eq!(map(press(KeyCode::F(1))), None);
        assert_eq!(map(press(KeyCode::Char('c'))), None);
    }

    #[test]
    fn non_press_events_are_ignored() {
        let mut event = press(KeyCode::Char('q'));
        event.kind = KeyEventKind::Release;
        assert_eq!(map(event), None);
    }
}
