use crate::{
    display::get_elapsed_millis,
    key::AccountKey,
    node_account::{AccountDisplay, NodeAccount},
    staking::{StakeLedger, StakeOverview},
};
use ratatui::{layout::Alignment, text::Text, widgets::Row};
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

type Commission = u32;
type Points = u32;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ValidatorStatus {
    /// Validator is an authority in the active set, displayed as [A]
    Authority,
    /// Validator is an authority and also a parachain authority, displayed as [P]
    ParaAuthority,
    /// Validator is in the waiting queue, displayed as [W]
    #[default]
    Waiting,
    /// Validator status is unknown or not yet determined, displayed as [U]
    Unknown,
}

impl std::fmt::Display for ValidatorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authority => write!(f, "[A]"),
            Self::ParaAuthority => write!(f, "[P]"),
            Self::Waiting => write!(f, "[W]"),
            Self::Unknown => write!(f, "[U]"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Nominators {
    pub stash: AccountId32,
    pub stake: u128,
    pub is_backer: bool,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub account: NodeAccount,
    pub commission: Commission,
    pub stake: StakeOverview,
    pub ledger: StakeLedger,
    pub nominators: Vec<Nominators>,
    // Track session points from staking_ah_client.validator_points
    pub points: Points,
    // Track old points so it can be better rendered the delta points
    pub old_points: Points,
    pub old_points_ts: u128,
    // Track era points accumulated at every new session from staking.era_reward_points
    // the total points earned at any single time will be sum of points + era_points
    pub era_points: Points,
    pub is_next_authority: bool,
    pub is_chilled: bool,
    pub status: ValidatorStatus,
}

impl Validator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
            commission: 0,
            stake: StakeOverview::default(),
            ledger: StakeLedger::default(),
            nominators: Vec::new(),
            points: 0,
            old_points: 0,
            old_points_ts: 0,
            era_points: 0,
            is_next_authority: false,
            is_chilled: false,
            status: ValidatorStatus::default(),
        }
    }

    pub fn key(&self) -> &AccountKey {
        &self.account.account_key()
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.account.runtime()
    }

    pub fn identity(&self) -> &Option<String> {
        self.account.identity()
    }

    pub fn display_name(&self) -> String {
        if let Some(display_name) = self.identity() {
            display_name.clone()
        } else {
            self.to_compact_string(6)
        }
    }

    pub fn commission_as_percentage(&self, decimal_places: usize) -> String {
        let percentage = self.commission as f64 / 10_000_000.0;
        let formatted = format!("{:.prec$}", percentage, prec = decimal_places);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        format!("{}%", trimmed)
    }

    pub fn points(&self) -> Points {
        self.points
    }

    pub fn total_points(&self) -> Points {
        self.points + self.era_points
    }

    pub fn delta_points(&self) -> Option<Points> {
        if self.points <= self.old_points {
            return None;
        }
        let elapsed = get_elapsed_millis(self.old_points_ts);
        if elapsed >= 2_000 {
            return None;
        }
        return Some(self.points - self.old_points);
    }

    pub fn status(&self) -> &ValidatorStatus {
        &self.status
    }

    pub fn is_active(&self) -> bool {
        self.status == ValidatorStatus::Authority || self.status == ValidatorStatus::ParaAuthority
    }

    pub fn is_waiting(&self) -> bool {
        self.status == ValidatorStatus::Waiting
    }

    pub fn is_unknown(&self) -> bool {
        self.status == ValidatorStatus::Unknown
    }
}

impl AccountDisplay for Validator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }
}

impl From<&Validator> for Row<'_> {
    fn from(v: &Validator) -> Self {
        // TODO: Verify if proxy is available and correctly setup for each stash
        let has_proxy = false;
        let status = if has_proxy { "[P]" } else { "[R]" };
        let v = v.clone();
        Row::new(vec![
            Text::from(format!("{}/{}", v.runtime(), v.display_name(),)),
            Text::from(status).alignment(Alignment::Right),
        ])
    }
}
