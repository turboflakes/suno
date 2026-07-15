use crate::app::Focus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use suno_actions::{Action, InputAction, NavigationAction, PopupAction, SystemAction};

/// Handles the key events and triggers respective action.
pub fn handle_key_events(key_event: KeyEvent, app_focus: Focus) -> Action {
    // Check for pressed ctrl + shift + key combination first
    if key_event
        .modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    {
        return match key_event.code {
            // Select previous main window/tab
            KeyCode::Char('w') => Action::Navigation(NavigationAction::PrevWindow),
            KeyCode::Char('c') => Action::Navigation(NavigationAction::Copy),
            _ => Action::System(SystemAction::Noop),
        };
    }
    // Check for pressed ctrl + key
    match key_event.modifiers {
        KeyModifiers::CONTROL => {
            match key_event.code {
                // Exit application on ctrl-c`
                KeyCode::Char('c') => Action::System(SystemAction::Quit),
                // Open popup on `ctrl-e` within the active section
                KeyCode::Char('e') => Action::Popup(PopupAction::Open),
                // Select next main window/tab
                KeyCode::Char('w') => Action::Navigation(NavigationAction::NextWindow),
                // Mask/Unmask Host IP addresses
                KeyCode::Char('m') => Action::Navigation(NavigationAction::ToggleMask),
                _ => match app_focus {
                    Focus::Main => match key_event.code {
                        KeyCode::Char('h') => Action::Navigation(NavigationAction::SectionUp),
                        KeyCode::Char('j') => Action::Navigation(NavigationAction::MoveUp),
                        KeyCode::Char('k') => Action::Navigation(NavigationAction::MoveDown),
                        KeyCode::Char('l') => Action::Navigation(NavigationAction::SectionDown),
                        _ => Action::System(SystemAction::Noop),
                    },
                    Focus::Input => match key_event.code {
                        KeyCode::Char('j') => Action::Navigation(NavigationAction::MoveUp),
                        KeyCode::Char('k') => Action::Navigation(NavigationAction::MoveDown),
                        _ => Action::System(SystemAction::Noop),
                    },
                    _ => Action::System(SystemAction::Noop),
                },
            }
        }
        _ => handle_key_events_without_modifiers(key_event, app_focus),
    }
}

fn handle_key_events_without_modifiers(key_event: KeyEvent, app_focus: Focus) -> Action {
    match app_focus {
        Focus::Main => handle_main_key_events(key_event),
        Focus::Popup | Focus::Scanner => handle_popup_key_events(key_event),
        Focus::Input => handle_editing_key_events(key_event),
        // _ => Action::System(SystemAction::Noop),
    }
}

fn handle_main_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        // Section Down on `Right`
        KeyCode::Right | KeyCode::Tab => Action::Navigation(NavigationAction::SectionDown),
        // Section Up on `Left`
        KeyCode::Left | KeyCode::BackTab => Action::Navigation(NavigationAction::SectionUp),
        // Move Up on `Up` inside the active section or list
        KeyCode::Up => Action::Navigation(NavigationAction::MoveUp),
        // Move Down on `Down` inside the active section or list
        KeyCode::Down => Action::Navigation(NavigationAction::MoveDown),
        // Reset active selections
        KeyCode::Esc => Action::Navigation(NavigationAction::Reset),
        _ => Action::System(SystemAction::Noop),
    }
}

fn handle_popup_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        // KeyCode::Up | KeyCode::Char('j') => Action::Navigation(NavigationAction::MoveUp),
        // KeyCode::Down | KeyCode::Char('k') => Action::Navigation(NavigationAction::MoveDown),
        // TODO: Implement KeyBinddings dynamically from config.
        // KeyCode::Char('c') => Action::Validator(ValidatorAction::SubmitChill),
        // KeyCode::Char('b') => Action::Validator(ValidatorAction::SubmitBond),
        // KeyCode::Char('u') => Action::Validator(ValidatorAction::SubmitUnbond),
        // KeyCode::Char('r') => Action::Validator(ValidatorAction::SubmitChangeRewardDestination),
        // KeyCode::Char('f') => Action::Validator(ValidatorAction::SubmitChangeCommission),
        // KeyCode::Char('k') => Action::Validator(ValidatorAction::SubmitKickNominators),
        // KeyCode::Char('s') => Action::Validator(ValidatorAction::SubmitSetSessionKey),
        KeyCode::Tab => Action::Input(InputAction::Editing),
        KeyCode::Esc => Action::Popup(PopupAction::Cancel),
        _ => Action::System(SystemAction::Noop),
    }
}

fn handle_editing_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        KeyCode::Tab => Action::Input(InputAction::AutoComplete),
        KeyCode::Char(c) => Action::Input(InputAction::Char(c)),
        KeyCode::Enter => Action::Input(InputAction::Enter),
        KeyCode::Backspace => Action::Input(InputAction::Delete),
        KeyCode::Left => Action::Input(InputAction::CursorLeft),
        KeyCode::Right => Action::Input(InputAction::CursorRight),
        KeyCode::Esc => Action::Input(InputAction::Unfocus),
        KeyCode::Up => Action::Navigation(NavigationAction::MoveUp),
        KeyCode::Down => Action::Navigation(NavigationAction::MoveDown),
        _ => Action::System(SystemAction::Noop),
    }
}
