use crate::{
    babe::Epoch,
    staking::{Era, StakeLedger},
    validator::ValidatorStatus,
};
use sp_arithmetic::Permill;
use std::fmt::Debug;

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

/// Stake ledger data combining account and ledger
#[derive(Debug)]
pub struct StakeLedgerData {
    pub account: AccountBytes,
    pub ledger: Option<StakeLedger>,
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
    StakeLedger(Data<StakeLedgerData>),
    ActiveValidators(Data<u32>),
    ActiveNominators(Data<u32>),
    TotalValidators(Data<u32>),
    TotalNominators(Data<u32>),
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
}
