use crate::app::Focus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use suno_actions::{
    Action, InputAction, NavigationAction, PopupAction, SystemAction, ValidatorAction,
};

/// Handles the key events and triggers respective action.
pub fn handle_key_events(key_event: KeyEvent, app_focus: Focus) -> Action {
    match key_event.modifiers {
        KeyModifiers::CONTROL => {
            match key_event.code {
                // Exit application on `Ctrl-C`
                KeyCode::Char('c') | KeyCode::Char('C') => Action::System(SystemAction::Quit),
                _ => Action::System(SystemAction::Noop),
            }
        }
        // TODO: It seems that `command` key on macos is not implemented for SUPER key modifiers
        KeyModifiers::ALT => match key_event.code {
            KeyCode::Tab => Action::Navigation(NavigationAction::NextTab),
            _ => Action::System(SystemAction::Noop),
        },
        _ => handle_key_events_without_modifiers(key_event, app_focus),
    }
}

fn handle_key_events_without_modifiers(key_event: KeyEvent, app_focus: Focus) -> Action {
    match app_focus {
        Focus::Main => handle_main_key_events(key_event),
        Focus::Popup => handle_popup_key_events(key_event),
        Focus::Input => handle_editing_key_events(key_event),
    }
}

fn handle_main_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        // Open popup menu within the active section
        KeyCode::Char('x') | KeyCode::Char('X') => Action::Popup(PopupAction::Open),
        // Section Down on `Right`
        KeyCode::Right | KeyCode::Tab => Action::Navigation(NavigationAction::SectionDown),
        // Section Up on `Left`
        KeyCode::Left | KeyCode::BackTab => Action::Navigation(NavigationAction::SectionUp),
        // Move Up on `Up` inside the active section or list
        KeyCode::Up => Action::Navigation(NavigationAction::MoveUp),
        // Move Down on `Down` inside the active section or list
        KeyCode::Down => Action::Navigation(NavigationAction::MoveDown),
        // Fallback to PrevTab
        KeyCode::Char('[') => Action::Navigation(NavigationAction::PrevTab),
        // Fallback to NextTab
        KeyCode::Char(']') => Action::Navigation(NavigationAction::NextTab),
        _ => Action::System(SystemAction::Noop),
    }
}

fn handle_popup_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        // Close popup menu
        KeyCode::Char('x') | KeyCode::Char('X') => Action::Popup(PopupAction::Close),
        KeyCode::Tab => Action::Input(InputAction::Editing),
        KeyCode::Up => Action::Navigation(NavigationAction::MoveUp),
        KeyCode::Down => Action::Navigation(NavigationAction::MoveDown),
        // TODO: Implement KeyBinddings dynamically from config.
        KeyCode::Char('c') => Action::Validator(ValidatorAction::SubmitChill),
        KeyCode::Char('b') => Action::Validator(ValidatorAction::SubmitBond),
        KeyCode::Char('u') => Action::Validator(ValidatorAction::SubmitUnbond),
        KeyCode::Char('r') => Action::Validator(ValidatorAction::SubmitChangeRewardDestination),
        KeyCode::Char('f') => Action::Validator(ValidatorAction::SubmitChangeCommission),
        KeyCode::Char('k') => Action::Validator(ValidatorAction::SubmitKickNominators),
        KeyCode::Char('s') => Action::Validator(ValidatorAction::SubmitSetSessionKey),
        KeyCode::Enter => Action::Popup(PopupAction::Confirm),
        KeyCode::Esc => Action::Popup(PopupAction::Cancel),
        _ => Action::System(SystemAction::Noop),
    }
}

fn handle_editing_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        KeyCode::Enter => Action::Input(InputAction::Submit),
        KeyCode::Char(letter) => Action::Input(InputAction::Char(letter)),
        KeyCode::Backspace => Action::Input(InputAction::Delete),
        KeyCode::Left => Action::Input(InputAction::CursorLeft),
        KeyCode::Right => Action::Input(InputAction::CursorRight),
        KeyCode::Esc => Action::Input(InputAction::Unfocus),
        _ => Action::System(SystemAction::Noop),
    }
}
