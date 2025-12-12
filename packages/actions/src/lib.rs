pub mod network;

use crate::network::ConnectionState;
use suno_config::SupportedRuntime;
use suno_primitives::AccountKey;

type Commission = u32;
type Points = u32;

/// Application actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Navigation actions
    Navigation(NavigationAction),
    /// Popup actions
    Popup(PopupAction),
    /// Network related actions
    Chain(ChainAction),
    /// Validator actions
    Validator(ValidatorAction),
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
    FetchInitialValidatorData(AccountKey),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidatorAction {
    SubmitChill,
    SubmitBond,
    SubmitUnbond,
    SubmitChangeRewardDestination,
    SubmitChangeCommission,
    SubmitKickNominators,
    SubmitSetSessionKey,
    UpdateChangeCommission(AccountKey, Commission),
    UpdatePoints(AccountKey, Points),
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
