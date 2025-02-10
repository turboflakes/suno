use crate::app::Action;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and triggers respective action.
pub fn handle_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => Action::Quit,
        // Exit application on `Ctrl-C`
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if key_event.modifiers == KeyModifiers::CONTROL {
                Action::Quit
            } else {
                Action::Noop
            }
        }
        // Scroll Up on `Option-Up`
        KeyCode::Up => {
            if key_event.modifiers == KeyModifiers::ALT {
                Action::ScrollUp
            } else {
                Action::SectionUp
            }
        }
        // Scroll Down on `Option-Down`
        KeyCode::Down => {
            if key_event.modifiers == KeyModifiers::ALT {
                Action::ScrollDown
            } else {
                Action::SectionDown
            }
        }
        _ => Action::Noop,
    }
}
