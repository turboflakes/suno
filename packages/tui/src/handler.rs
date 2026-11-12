use crate::app::Focus;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use suno_actions::{
    Action, FilterAction, InputAction, NavigationAction, PopupAction, SystemAction,
};
use suno_primitives::call::Call;

/// Handles the key events and triggers respective action.
pub fn handle_key_events(key_event: KeyEvent, app_focus: Focus) -> Action {
    // The key after `ctrl+x` completes the chord, anything else ends it
    if app_focus == Focus::Chord
        && !key_event
            .modifiers
            .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT)
    {
        return handle_chord_key_events(key_event);
    }
    // Check for pressed ctrl + shift + key combination first
    if key_event
        .modifiers
        .contains(KeyModifiers::CONTROL | KeyModifiers::SHIFT)
    {
        return match key_event.code {
            // Select previous main window/tab
            KeyCode::Char('w') => Action::Navigation(NavigationAction::PrevWindow),
            KeyCode::Char('c' | 'C') => Action::Navigation(NavigationAction::Copy),
            _ => Action::System(SystemAction::Noop),
        };
    }
    // Check for pressed ctrl + key
    match key_event.modifiers {
        KeyModifiers::CONTROL => {
            match key_event.code {
                // Exit application on ctrl-c`
                KeyCode::Char('c') => Action::System(SystemAction::Quit),
                // Update application on ctrl-u`
                KeyCode::Char('u') => Action::System(SystemAction::Update),
                // Start the chord to add an account, completed by the next key
                KeyCode::Char('x') if matches!(app_focus, Focus::Main | Focus::None) => {
                    Action::Navigation(NavigationAction::StartChord)
                }
                // Open popup on `ctrl-e` within the active section
                KeyCode::Char('e') => Action::Popup(PopupAction::Open),
                // Open the theme selection popup
                KeyCode::Char('t') => Action::Popup(PopupAction::OpenThemeMenu),
                // Open the features popup, to switch them on and off
                KeyCode::Char('o') => Action::Popup(PopupAction::OpenFeatures),
                // Filter the list of validators or collators displayed
                KeyCode::Char('f') => Action::Filter(FilterAction::Open),
                // Select next main window/tab
                KeyCode::Char('w') => Action::Navigation(NavigationAction::NextWindow),
                // Mask/Unmask Host IP addresses
                KeyCode::Char('m') => Action::Navigation(NavigationAction::ToggleMask),
                // Paste the clipboard while typing, instead of toggling the view
                KeyCode::Char('v') if app_focus == Focus::Input => {
                    Action::Input(InputAction::PasteClipboard)
                }
                // Toggle between grouped and listed detailed views
                KeyCode::Char('v') => Action::Navigation(NavigationAction::ToggleView),
                _ => match app_focus {
                    Focus::Main | Focus::None => match key_event.code {
                        KeyCode::Char('h') => Action::Navigation(NavigationAction::PrevTab),
                        KeyCode::Char('j') => Action::Navigation(NavigationAction::MoveUp),
                        KeyCode::Char('k') => Action::Navigation(NavigationAction::MoveDown),
                        KeyCode::Char('l') => Action::Navigation(NavigationAction::NextTab),
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

/// The keys that complete the chord started with `ctrl+x`
fn handle_chord_key_events(key_event: KeyEvent) -> Action {
    let call = match key_event.code {
        // Copy the selected address, same as ctrl+shift+c
        KeyCode::Char('a') => return Action::Navigation(NavigationAction::Copy),
        KeyCode::Char('v') => Call::AddValidator { stash: None },
        KeyCode::Char('n') => Call::AddNominator { stash: None },
        KeyCode::Char('c') => Call::AddCollator { stash: None },
        KeyCode::Char('p') => Call::AddProxyAccount { account: None },
        _ => return Action::Navigation(NavigationAction::EndChord),
    };
    Action::Popup(PopupAction::OpenCall(Box::new(call)))
}

fn handle_key_events_without_modifiers(key_event: KeyEvent, app_focus: Focus) -> Action {
    match app_focus {
        Focus::Main | Focus::None => handle_main_key_events(key_event),
        Focus::Popup | Focus::Scanner => handle_popup_key_events(key_event),
        Focus::Input => handle_editing_key_events(key_event),
        Focus::Filter => handle_filter_key_events(key_event),
        Focus::Chord => Action::System(SystemAction::Noop),
        // _ => Action::System(SystemAction::Noop),
    }
}

fn handle_main_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        // Section Down on `Tab`
        KeyCode::Tab => Action::Navigation(NavigationAction::SectionDown),
        // Section Up on `BackTab`
        KeyCode::BackTab => Action::Navigation(NavigationAction::SectionUp),
        // Focus the window tabs, and then the next window, on `Right`
        KeyCode::Right => Action::Navigation(NavigationAction::NextTab),
        // Focus the window tabs, and then the previous window, on `Left`
        KeyCode::Left => Action::Navigation(NavigationAction::PrevTab),
        // Extend the selection on `Shift+Up`
        KeyCode::Up if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
            Action::Navigation(NavigationAction::ExtendUp)
        }
        // Extend the selection on `Shift+Down`
        KeyCode::Down if key_event.modifiers.contains(KeyModifiers::SHIFT) => {
            Action::Navigation(NavigationAction::ExtendDown)
        }
        // Move Up on `Up` inside the active section or list
        KeyCode::Up => Action::Navigation(NavigationAction::MoveUp),
        // Move Down on `Down` inside the active section or list
        KeyCode::Down => Action::Navigation(NavigationAction::MoveDown),
        // Typing letters or digits starts the filter of the validators or the collators
        KeyCode::Char(c)
            if c.is_alphanumeric()
                && !key_event
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            Action::Filter(FilterAction::Start(c))
        }
        // Pin or unpin the selected validator or collator on `Space`
        KeyCode::Char(' ') => Action::Navigation(NavigationAction::TogglePin),
        // Open popup within the active section on `/`, same as on `ctrl-e`
        KeyCode::Char('/') => Action::Popup(PopupAction::Open),
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
        KeyCode::Tab => Action::Input(InputAction::Editing(None)),
        // Typing letters or digits, focus and starts input editing
        KeyCode::Char(c)
            if c.is_alphanumeric()
                && !key_event
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            Action::Input(InputAction::Editing(Some(c)))
        }
        // The chain of the commands shown is switched without focusing the input
        KeyCode::Left => Action::Input(InputAction::CursorLeft),
        KeyCode::Right => Action::Input(InputAction::CursorRight),
        KeyCode::Up => Action::Navigation(NavigationAction::MoveUp),
        KeyCode::Down => Action::Navigation(NavigationAction::MoveDown),
        KeyCode::Char(' ') => Action::Popup(PopupAction::ToggleFeature),
        KeyCode::Enter => Action::Popup(PopupAction::Confirm),
        KeyCode::Char('r') => Action::Popup(PopupAction::ResetFeature),
        KeyCode::Esc => Action::Popup(PopupAction::Cancel),
        _ => Action::System(SystemAction::Noop),
    }
}

fn handle_filter_key_events(key_event: KeyEvent) -> Action {
    match key_event.code {
        KeyCode::Char(c) => Action::Filter(FilterAction::Char(c)),
        KeyCode::Backspace => Action::Filter(FilterAction::Delete),
        KeyCode::Left => Action::Filter(FilterAction::CursorLeft),
        KeyCode::Right => Action::Filter(FilterAction::CursorRight),
        // Leave the input to the list, where the first row gets highlighted
        KeyCode::Up | KeyCode::Down | KeyCode::Enter => Action::Filter(FilterAction::Select),
        KeyCode::Esc => Action::Filter(FilterAction::Clear),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn key(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent::new(code, modifiers)
    }

    #[test]
    fn ctrl_x_starts_the_chord_only_when_nothing_has_the_keys() {
        let ctrl_x = key(KeyCode::Char('x'), KeyModifiers::CONTROL);
        for focus in [Focus::Main, Focus::None] {
            assert_eq!(
                handle_key_events(ctrl_x, focus),
                Action::Navigation(NavigationAction::StartChord)
            );
        }
        assert_eq!(
            handle_key_events(ctrl_x, Focus::Input),
            Action::System(SystemAction::Noop)
        );
    }

    #[test]
    fn the_key_after_the_chord_adds_an_account_or_ends_it() {
        let plain = |c| key(KeyCode::Char(c), KeyModifiers::NONE);
        assert_eq!(
            handle_key_events(plain('n'), Focus::Chord),
            Action::Popup(PopupAction::OpenCall(Box::new(Call::AddNominator {
                stash: None
            })))
        );
        assert_eq!(
            handle_key_events(plain('z'), Focus::Chord),
            Action::Navigation(NavigationAction::EndChord)
        );
        assert_eq!(
            handle_key_events(key(KeyCode::Esc, KeyModifiers::NONE), Focus::Chord),
            Action::Navigation(NavigationAction::EndChord)
        );
    }
}
