use crate::{
    babe::Epoch,
    staking::{Era, StakeLedger},
    validator::ValidatorStatus,
};
use sp_arithmetic::Permill;
use std::fmt::Debug;

type AccountBytes = [u8; 32];

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
    StakeLedger(Data<StakeLedgerData>),
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

    pub fn stake_ledger(account: AccountBytes, ledger: Option<StakeLedger>) -> Self {
        Response::StakeLedger(Data::new(StakeLedgerData { account, ledger }))
    }

    pub fn total_validators(value: u32) -> Self {
        Response::TotalValidators(Data::new(value))
    }

    pub fn total_nominators(value: u32) -> Self {
        Response::TotalNominators(Data::new(value))
    }
}
