pub mod network;

use crate::network::ConnectionState;
use snops_config::SupportedRuntime;

/// Application actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Navigation actions
    Navigation(NavigationAction),
    /// Popup actions
    Popup(PopupAction),
    /// Network related actions
    Chain(ChainAction),
    /// Staking actions
    Staking(StakingAction),
    /// Transaction related actions
    Transaction(TxAction),
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
    NextTab,
    PrevTab,
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
pub enum TxAction {
    Broadcasting,
    InBestBlock,
    InFinalizedBlock,
    Success,
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SystemAction {
    Quit,
    Tick,
    Noop,
    Error(String),
}
