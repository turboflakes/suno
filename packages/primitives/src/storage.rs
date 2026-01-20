use crate::{
    babe::Epoch,
    identity::Identity,
    staking::{Era, StakeLedger, StakeOverview},
    validator::ValidatorStatus,
};
use sp_arithmetic::{Perbill, Permill};
use std::fmt::Debug;
use subxt::{
    error::TransactionError,
    tx::TxProgress,
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};

type AccountBytes = [u8; 32];
type Points = u32;

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

/// Commission data combining account and commission
#[derive(Debug)]
pub struct CommissionData {
    pub account: AccountBytes,
    pub commission: Perbill,
}

/// Stake ledger data combining account and ledger
#[derive(Debug)]
pub struct IdentityData {
    pub account: AccountBytes,
    pub identity: Option<Identity>,
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
    Commission(Data<CommissionData>),
    Identity(Data<IdentityData>),
    ActiveValidators(Data<u32>),
    ActiveNominators(Data<u32>),
    TotalValidators(Data<u32>),
    TotalNominators(Data<u32>),
    TxProgress(Data<TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>>),
    TxSuccess,
    TxError(String),
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

    pub fn validator_commission(account: AccountBytes, commission: Perbill) -> Self {
        Response::Commission(Data::new(CommissionData {
            account,
            commission,
        }))
    }

    pub fn identity(account: AccountBytes, identity: Option<Identity>) -> Self {
        Response::Identity(Data::new(IdentityData { account, identity }))
    }

    pub fn transaction_progress(
        progress: TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    ) -> Self {
        Response::TxProgress(Data::new(progress))
    }
}
