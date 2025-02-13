use crate::actions::{Action, NavigationAction, PopupAction, StakingAction, SystemAction};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::info;

/// Handles the key events and triggers respective action.
pub fn handle_key_events(key_event: KeyEvent) -> Action {
    match key_event.modifiers {
        KeyModifiers::CONTROL => {
            match key_event.code {
                // Exit application on `Ctrl-C`
                KeyCode::Char('c') | KeyCode::Char('C') => Action::System(SystemAction::Quit),
                _ => Action::System(SystemAction::Noop),
            }
        }
        _ => handle_key_events_without_modifiers(key_event),
    }
}

pub fn handle_key_events_without_modifiers(key_event: KeyEvent) -> Action {
    match key_event.code {
        // Open popup menu within the active section
        KeyCode::Char('x') | KeyCode::Char('X') => Action::Popup(PopupAction::Toggle),
        // Section Up on `Left`
        KeyCode::Left | KeyCode::BackTab => Action::Navigation(NavigationAction::SectionUp),
        // Section Down on `Right`
        KeyCode::Right | KeyCode::Tab => Action::Navigation(NavigationAction::SectionDown),
        // Move Up on `Up`
        KeyCode::Up => Action::Navigation(NavigationAction::MoveUp),
        // Move Down on `Down`
        KeyCode::Down => Action::Navigation(NavigationAction::MoveDown),
        // TODO: Implement KeyBinddings dynamically from config.
        KeyCode::Char('c') => Action::Staking(StakingAction::Chill),
        KeyCode::Char('b') => Action::Staking(StakingAction::Bond),
        KeyCode::Char('u') => Action::Staking(StakingAction::Unbond),
        KeyCode::Char('r') => Action::Staking(StakingAction::ChangeRewardDestination),
        KeyCode::Char('f') => Action::Staking(StakingAction::ChangeCommission),
        KeyCode::Char('k') => Action::Staking(StakingAction::KickNominators),
        KeyCode::Char('s') => Action::Staking(StakingAction::SetSessionKey),
        KeyCode::Char('y') | KeyCode::Enter => Action::Popup(PopupAction::Confirm),
        KeyCode::Char('n') | KeyCode::Esc => Action::Popup(PopupAction::Cancel),
        _ => Action::System(SystemAction::Noop),
    }
}
