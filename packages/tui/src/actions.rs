use crate::config::SupportedRuntime;
use crate::widgets::chains::ConnectionState;

/// Application actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Navigation actions
    Navigation(NavigationAction),
    /// Popup actions
    Popup(PopupAction),
    /// Chain-related actions
    Chain(ChainAction),
    /// Staking actions
    Staking(StakingAction),
    //TODO: Collator actions
    // Collator(CollatorAction),
    /// System actions
    System(SystemAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationAction {
    MoveUp,
    MoveDown,
    SectionUp,
    SectionDown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupAction {
    Toggle,
    Confirm,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainAction {
    Connection {
        runtime: SupportedRuntime,
        state: ConnectionState,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StakingAction {
    Chill,
    Bond,
    Unbond,
    ChangeRewardDestination,
    ChangeCommission,
    KickNominators,
    SetSessionKey,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemAction {
    Quit,
    Tick,
    Noop,
    Error(String),
}
