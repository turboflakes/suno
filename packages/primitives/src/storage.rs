use crate::{
    babe::Epoch,
    identity::Identity,
    staking::{Chunk, Era, Payee, StakeLedger, StakeOverview, ValidatorPrefs},
    validator::ValidatorStatus,
};
use sp_arithmetic::Permill;
use std::fmt::Debug;
use subxt::{tx::TxProgress, OnlineClient, SubstrateConfig};

type AccountBytes = [u8; 32];
type Points = u32;
type Amount = u128;

/// Generic data structure for chain responses
#[derive(Debug)]
pub struct Data<T: Debug> {
    pub value: T,
}

impl<T: Debug> Data<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

/// Authority status data combining account and status
#[derive(Debug)]
pub struct AuthorityStatus {
    pub account: AccountBytes,
    pub status: ValidatorStatus,
}

#[derive(Debug)]
pub struct AuthorityPoints {
    pub account: AccountBytes,
    pub points: Points,
}

/// Stake overview data combining account and overview
#[derive(Debug)]
pub struct StakeOverviewData {
    pub account: AccountBytes,
    pub overview: Option<StakeOverview>,
}

/// Stake ledger data combining account and ledger
#[derive(Debug)]
pub struct StakeLedgerData {
    pub account: AccountBytes,
    pub ledger: Option<StakeLedger>,
}

/// Validator prefs data combining account and prefs
#[derive(Debug)]
pub struct ValidatorPrefsData {
    pub account: AccountBytes,
    pub prefs: Option<ValidatorPrefs>,
}

/// Stake ledger data combining account and ledger
#[derive(Debug)]
pub struct IdentityData {
    pub account: AccountBytes,
    pub identity: Option<Identity>,
}

/// Amount data combining account and amount
#[derive(Debug)]
pub struct AmountData {
    pub account: AccountBytes,
    pub amount: Amount,
}

/// Chunk data combining account and chunk
#[derive(Debug)]
pub struct ChunkData {
    pub account: AccountBytes,
    pub chunk: Chunk,
}

/// Chunk data combining account and chunk
#[derive(Debug)]
pub struct ValidatorPayeeData {
    pub account: AccountBytes,
    pub payee: Payee,
}

/// Response types from chain storage queries
/// This enum allows heterogeneous collection of different data types
#[derive(Debug)]
pub enum Response {
    Era(Data<Era>),
    Epoch(Data<Epoch>),
    TotalStaked(Data<Permill>),
    AuthorityStatus(Data<AuthorityStatus>),
    AuthorityEraPoints(Data<AuthorityPoints>),
    AuthorityPoints(Data<AuthorityPoints>),
    StakeOverview(Data<StakeOverviewData>),
    StakeLedger(Data<StakeLedgerData>),
    ValidatorPrefs(Data<ValidatorPrefsData>),
    ValidatorPrefsNext(Data<ValidatorPrefsData>),
    ValidatorPayee(Data<ValidatorPayeeData>),
    Identity(Data<IdentityData>),
    ActiveValidators(Data<u32>),
    ActiveNominators(Data<u32>),
    TotalValidators(Data<u32>),
    TotalNominators(Data<u32>),
    TxProgress(Data<TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>>),
    TxSuccess,
    TxError(String),
    EventBonded(Data<AmountData>),
    EventUnbonded(Data<ChunkData>),
}

// Some constructors for convenience

impl Response {
    pub fn era(era: Era) -> Self {
        Response::Era(Data::new(era))
    }

    pub fn epoch(epoch: Epoch) -> Self {
        Response::Epoch(Data::new(epoch))
    }

    pub fn total_staked(value: Permill) -> Self {
        Response::TotalStaked(Data::new(value))
    }

    pub fn authority_status(account: AccountBytes, status: ValidatorStatus) -> Self {
        Response::AuthorityStatus(Data::new(AuthorityStatus { account, status }))
    }

    pub fn authority_era_points(account: AccountBytes, points: Points) -> Self {
        Response::AuthorityEraPoints(Data::new(AuthorityPoints { account, points }))
    }

    pub fn authority_points(account: AccountBytes, points: Points) -> Self {
        Response::AuthorityPoints(Data::new(AuthorityPoints { account, points }))
    }

    pub fn stake_overview(account: AccountBytes, overview: Option<StakeOverview>) -> Self {
        Response::StakeOverview(Data::new(StakeOverviewData { account, overview }))
    }

    pub fn stake_ledger(account: AccountBytes, ledger: Option<StakeLedger>) -> Self {
        Response::StakeLedger(Data::new(StakeLedgerData { account, ledger }))
    }

    pub fn active_validators(value: u32) -> Self {
        Response::ActiveValidators(Data::new(value))
    }

    pub fn active_nominators(value: u32) -> Self {
        Response::ActiveNominators(Data::new(value))
    }

    pub fn total_validators(value: u32) -> Self {
        Response::TotalValidators(Data::new(value))
    }

    pub fn total_nominators(value: u32) -> Self {
        Response::TotalNominators(Data::new(value))
    }

    pub fn validator_prefs(account: AccountBytes, prefs: Option<ValidatorPrefs>) -> Self {
        Response::ValidatorPrefs(Data::new(ValidatorPrefsData { account, prefs }))
    }

    pub fn validator_prefs_next(account: AccountBytes, prefs: Option<ValidatorPrefs>) -> Self {
        Response::ValidatorPrefsNext(Data::new(ValidatorPrefsData { account, prefs }))
    }

    pub fn validator_payee(account: AccountBytes, payee: Payee) -> Self {
        Response::ValidatorPayee(Data::new(ValidatorPayeeData { account, payee }))
    }

    pub fn identity(account: AccountBytes, identity: Option<Identity>) -> Self {
        Response::Identity(Data::new(IdentityData { account, identity }))
    }

    pub fn transaction_progress(
        progress: TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    ) -> Self {
        Response::TxProgress(Data::new(progress))
    }

    pub fn event_bonded(account: AccountBytes, amount: Amount) -> Self {
        Response::EventBonded(Data::new(AmountData { account, amount }))
    }

    pub fn event_unbonded(account: AccountBytes, chunk: Chunk) -> Self {
        Response::EventUnbonded(Data::new(ChunkData { account, chunk }))
    }
}
