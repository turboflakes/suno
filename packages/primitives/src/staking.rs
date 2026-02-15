use crate::display::format_millis;
use serde::Serialize;
use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};
use subxt::utils::AccountId32;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Era {
    // Era index
    index: u32,
    // Timestamp when the era started
    start_ts: u64,
    // Session that the era started
    start_session: u64,
    // Sessions per era
    sessions_per_era: u32,
}

impl Era {
    pub fn new(index: u32, start_ts: u64, start_session: u64, sessions_per_era: u32) -> Self {
        Self {
            index,
            start_ts,
            start_session,
            sessions_per_era,
        }
    }

    pub fn index(&self) -> u32 {
        self.index
    }

    pub fn duration(&self, blocks_per_session: u64) -> u64 {
        self.sessions_per_era as u64 * blocks_per_session
    }

    fn duration_ms(&self, blocks_per_session: u64, block_time_ms: u64) -> u64 {
        self.duration(blocks_per_session) * block_time_ms
    }

    pub fn progress(&self, blocks_per_session: u64, block_time_ms: u64) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let diff = now - self.start_ts as u128;
        diff as f64 / self.duration_ms(blocks_per_session, block_time_ms) as f64
    }

    pub fn countdown_time(&self, blocks_per_session: u64, block_time_ms: u64) -> String {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let duration = self.duration_ms(blocks_per_session, block_time_ms);

        if duration < (now - self.start_ts as u128) as u64 {
            return format_millis(0);
        }

        let diff = duration - (now - self.start_ts as u128) as u64;

        format_millis(diff)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct StakeOverview {
    pub own: u128,
    pub total: u128,
    pub nominators_count: u32,
}

impl StakeOverview {
    pub fn new(own: u128, total: u128, nominators_count: u32) -> Self {
        Self {
            own,
            total,
            nominators_count,
        }
    }

    pub fn own(&self) -> u128 {
        self.own
    }

    pub fn total(&self) -> u128 {
        self.total
    }

    pub fn nominators_count(&self) -> u32 {
        self.nominators_count
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Unlocking {
    pub era: u32,
    pub value: u128,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct StakeLedger {
    pub active: u128,
    pub total: u128,
    pub unlocking: Vec<Unlocking>,
}

impl StakeLedger {
    // TODO: add unlocking: Vec<Unlocking>
    pub fn new(active: u128, total: u128) -> Self {
        Self {
            active,
            total,
            unlocking: Vec::new(),
        }
    }

    pub fn active(&self) -> u128 {
        self.active
    }

    pub fn total(&self) -> u128 {
        self.total
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Payee {
    #[default]
    None,
    Staked,
    Stash,
    Controller,
    Account(AccountId32),
}

impl std::fmt::Display for Payee {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "none"),
            Self::Staked => write!(f, "staked"),
            Self::Stash { .. } => write!(f, "stash"),
            Self::Controller { .. } => write!(f, "controller"),
            Self::Account(account) => write!(f, "account {}", account),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PayeeError {
    #[error("Command must be: {0}")]
    UnknownArgument(String),
    #[error("Command must be: {0}")]
    InvalidAddress(String),
}

impl FromStr for Payee {
    type Err = PayeeError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Ok(Payee::None);
        }
        match s.split_once(' ') {
            None => match s {
                "staked" => Ok(Payee::Staked),
                "stash" => Ok(Payee::Stash),
                "controller" => Ok(Payee::Controller),
                _ => Err(PayeeError::UnknownArgument(
                    "staked|stash|controller|account <address>".to_string(),
                )),
            },
            Some((argument, account)) => match argument {
                "account" => {
                    let acc = AccountId32::from_str(account)
                        .map_err(|_| PayeeError::InvalidAddress(account.to_string()))?;
                    Ok(Payee::Account(acc))
                }
                _ => Err(PayeeError::UnknownArgument(
                    "staked|stash|controller|account <address>".to_string(),
                )),
            },
        }
    }
}
