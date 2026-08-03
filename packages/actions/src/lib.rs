pub mod network;

use crate::network::ConnectionState;
use bytes::Bytes;
use image::DynamicImage;
use sp_arithmetic::Permill;
use subxt::utils::H256;
use suno_config::SupportedRuntime;
use suno_primitives::{
    babe::Epoch,
    balance::Balance,
    call::Call,
    identity::Identity,
    proxy::ProxyKey,
    session::Keys,
    staking::{Chunk, Era, Payee, StakeLedger, StakeOverview, ValidatorPrefs},
    validator::ValidatorStatus,
    AccountKey,
};
use suno_update::{AssetName, Checksum, Release};

type ValidatorKey = AccountKey;
type ChainKey = SupportedRuntime;
type BlockNumber = u64;
type BlockHash = H256;
type Amount = u128;
type Points = u32;
type Counter = u32;

/// Application actions.
#[derive(Debug, Clone, PartialEq)]
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
    ///
    /// QrScanner actions
    Scanner(ScannerAction),
    /// System actions
    System(SystemAction),
    /// Update related actions
    Update(UpdateAction),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NavigationAction {
    MoveUp,
    MoveDown,
    SectionUp,
    SectionDown,
    NextWindow,
    PrevWindow,
    Reset,
    Copy,
    ToggleMask,
}

type SpecVersion = u32;
type ProxyIdentity = String;
type StashIdentity = String;
type CallDataBytes = Vec<u8>;
type QrBytes = Vec<u8>;
type QrSignature = Vec<u8>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfirmationContext {
    pub runtime: SupportedRuntime,
    pub spec_version: SpecVersion,
    pub proxy_identity: ProxyIdentity,
    pub stash_identity: StashIdentity,
    pub call: Call,
    pub call_data_bytes: CallDataBytes,
    pub qr_bytes: QrBytes,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PopupAction {
    Open,
    ConfirmAndSign(Box<ConfirmationContext>),
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
    Success(String),
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
    AddProxy(ValidatorKey, ProxyKey),
    UpdateBalance(ValidatorKey, Balance),
    AddAmountToBalance(ValidatorKey, Amount),
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
    Update,
    Tick,
    Noop,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScannerAction {
    Init,
    Decoded(QrSignature),
    Frame(DynamicImage),
    Error(String),
}

/// Thread action messages, useful to be used in private channels and controlling threads from the app,
/// eg. stopping a scanner thread.
pub enum ThreadAction {
    Stop,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateAction {
    Start,
    Download(Release),
    Validate(AssetName, Bytes, Checksum),
    Complete,
    Error,
}
