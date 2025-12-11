pub mod network;

use crate::network::ConnectionState;
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;
use suno_primitives::ValidatorKey;

type Commission = u32;
type Stash = AccountId32;

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
    FetchInitialValidatorData(ValidatorKey),
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
    UpdateChangeCommission(ValidatorKey, Commission),
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
