pub mod network;

use crate::network::ConnectionState;
use sp_arithmetic::Permill;
use subxt::utils::H256;
use suno_config::SupportedRuntime;
use suno_primitives::{
    babe::Epoch,
    identity::Identity,
    staking::{Era, StakeLedger, StakeOverview},
    validator::ValidatorStatus,
    AccountKey,
};

type Commission = u32;
type Points = u32;
type Counter = u32;

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

type ValidatorKey = AccountKey;
type ChainKey = SupportedRuntime;
type BlockNumber = u64;
type BlockHash = H256;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChainAction {
    UpdateConnectionState(ChainKey, ConnectionState),
    UpdateBestBlock(ChainKey, BlockNumber),
    UpdateFinalizedBlock(ChainKey, BlockNumber, BlockHash),
    UpdateEra(ChainKey, Era),
    UpdateEpoch(ChainKey, Epoch),
    UpdateTotalStaked(ChainKey, Permill),
    UpdateActiveValidators(ChainKey, Counter),
    UpdateTotalValidators(ChainKey, Counter),
    UpdateActiveNominators(ChainKey, Counter),
    UpdateTotalNominators(ChainKey, Counter),
    FetchValidatorData(ValidatorKey),
    FetchValidatorsData(SupportedRuntime, Vec<ValidatorKey>),
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
    UpdateCommission(ValidatorKey, Commission),
    UpdatePoints(ValidatorKey, Points),
    UpdateEraPoints(ValidatorKey, Points),
    UpdateIdentity(ValidatorKey, Identity),
    UpdateStakeOverview(ValidatorKey, StakeOverview),
    UpdateStakeLedger(ValidatorKey, StakeLedger),
    UpdateStatus(ValidatorKey, ValidatorStatus),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxAction {
    Processing,
    Sent,
    InBestBlock(BlockHash),
    InFinalizedBlock(BlockHash),
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
