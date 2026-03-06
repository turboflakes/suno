pub mod network;

use crate::network::ConnectionState;
use sp_arithmetic::Permill;
use subxt::utils::H256;
use suno_config::SupportedRuntime;
use suno_primitives::{
    babe::Epoch,
    call::Call,
    identity::Identity,
    session::Keys,
    staking::{Chunk, Era, Payee, StakeLedger, StakeOverview, ValidatorPrefs},
    validator::ValidatorStatus,
    AccountKey,
};

type ValidatorKey = AccountKey;
type ChainKey = SupportedRuntime;
type BlockNumber = u64;
type BlockHash = H256;
type Amount = u128;
type Points = u32;
type Counter = u32;
type IsValid = bool;

/// Application actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// Navigation actions
    Navigation(NavigationAction),
    /// Popup actions
    Popup(PopupAction),
    /// Input related actions
    Input(InputAction),
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
    Reset,
    Copy,
}

type SpecVersion = u32;
type ProxyIdentity = String;
type StashIdentity = String;
type Bytes = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupAction {
    Open,
    ConfirmAndSign(
        SupportedRuntime,
        SpecVersion,
        ProxyIdentity,
        StashIdentity,
        Box<Call>,
        Bytes,
    ),
    Close,
    Cancel,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputAction {
    Char(char),
    AutoComplete,
    Enter,
    Delete,
    CursorLeft,
    CursorRight,
    Editing,
    Unfocus,
    Lock,
    Paste(String),
    Error(String),
}

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
    UpdateValidatorPrefs(ValidatorKey, ValidatorPrefs),
    UpdateValidatorPrefsNext(ValidatorKey, ValidatorPrefs),
    UpdatePoints(ValidatorKey, Points),
    UpdateEraPoints(ValidatorKey, Points),
    UpdateIdentity(ValidatorKey, Identity),
    UpdateStakeOverview(ValidatorKey, StakeOverview),
    UpdateStakeLedger(ValidatorKey, StakeLedger),
    UpdatePayee(ValidatorKey, Payee),
    UpdateNextKeys(ValidatorKey, Option<Keys>),
    UpdateQueuedKeys(ValidatorKey, Option<Keys>),
    AddAmountToStakeLedger(ValidatorKey, Amount),
    SubChunkFromStakeLedger(ValidatorKey, Chunk),
    UpdateStatus(ValidatorKey, ValidatorStatus),
    UpdateProxyStatus(ValidatorKey, IsValid),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TxAction {
    Processing,
    Message(&'static str),
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
