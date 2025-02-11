use crate::app::Action;
use crate::section::Section;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

/// Handles the key events and triggers respective action.
pub fn handle_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        // Exit application on `ESC` or `q`
        KeyCode::Esc | KeyCode::Char('q') => Action::Quit,
        // // Exit application on `Ctrl-C`
        // KeyCode::Char('c') | KeyCode::Char('C') => {
        //     if key_event.modifiers == KeyModifiers::CONTROL {
        //         Action::Quit
        //     } else {
        //         Action::Noop
        //     }
        // }
        // Open popup menu within the active section
        KeyCode::Char('x') | KeyCode::Char('X') => Action::TogglePopup,
        // Section Up on `Left`
        KeyCode::Left => Action::SectionUp,
        // Section Down on `Right`
        KeyCode::Right => Action::SectionDown,
        // Move Up on `Up`
        KeyCode::Up => Action::MoveUp,
        // Move Down on `Down`
        KeyCode::Down => Action::MoveDown,
        KeyCode::Char('c') => Action::Chill,
        KeyCode::Char('b') => Action::Bond,
        KeyCode::Char('u') => Action::Unbond,
        KeyCode::Char('r') => Action::ChangeRewardDestination,
        KeyCode::Char('f') => Action::ChangeCommission,
        KeyCode::Char('k') => Action::KickNominators,
        KeyCode::Char('s') => Action::SetSessionKey,
        _ => Action::Noop,
    }
}
