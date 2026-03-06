use crate::{
    display::get_elapsed_millis,
    identity::Identity,
    key::AccountKey,
    node_account::{AccountDisplay, NodeAccount},
    session::Keys,
    staking::{Payee, StakeLedger, StakeOverview, ValidatorPrefs},
};
use ratatui::{layout::Alignment, text::Text, widgets::Row};
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nominators {
    pub stash: AccountId32,
    pub stake: u128,
    pub is_backer: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Validator {
    pub account: NodeAccount,
    pub prefs: ValidatorPrefs,
    pub prefs_next: ValidatorPrefs,
    pub stake: StakeOverview,
    pub ledger: StakeLedger,
    pub payee: Payee,
    pub next_keys: Option<Keys>,
    pub queued_keys: Option<Keys>,
    pub nominators: Vec<Nominators>,
    // Track session points from staking_ah_client.validator_points
    pub points: Points,
    // Track old points so it can be better rendered the delta points
    pub old_points: Points,
    pub old_points_ts: u128,
    // Track era points accumulated at every new session from staking.era_reward_points
    // the total points earned at any single time will be sum of points + era_points
    pub era_points: Points,
    pub is_chilled: bool,
    pub is_proxy_valid: bool,
    pub status: ValidatorStatus,
}

impl Validator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
            prefs: ValidatorPrefs::default(),
            prefs_next: ValidatorPrefs::default(),
            stake: StakeOverview::default(),
            ledger: StakeLedger::default(),
            payee: Payee::None,
            next_keys: None,
            queued_keys: None,
            nominators: Vec::new(),
            points: 0,
            old_points: 0,
            old_points_ts: 0,
            era_points: 0,
            is_chilled: false,
            is_proxy_valid: false,
            status: ValidatorStatus::default(),
        }
    }

    pub fn key(&self) -> &AccountKey {
        self.account.account_key()
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.account.runtime()
    }

    pub fn identity(&self) -> &Option<Identity> {
        self.account.identity()
    }

    pub fn display_name(&self, size: usize) -> String {
        if let Some(identity) = self.identity() {
            format!("{} ({})", identity, self.to_compact_string(size))
        } else {
            self.to_compact_string(size)
        }
    }

    pub fn display_identity(&self) -> String {
        if let Some(identity) = self.identity() {
            identity.to_string()
        } else {
            self.to_compact_string(6)
        }
    }

    pub fn commission_as_percentage(&self, decimal_places: usize) -> String {
        self.prefs.commission_as_percentage(decimal_places)
    }

    pub fn next_commission_as_percentage(&self, decimal_places: usize) -> String {
        self.prefs_next.commission_as_percentage(decimal_places)
    }

    pub fn is_commission_changed(&self) -> bool {
        self.prefs_next.commission() != self.prefs.commission()
    }

    pub fn is_next_keys_changed(&self) -> bool {
        self.next_keys != self.queued_keys
    }

    pub fn is_next_authority(&self) -> bool {
        self.queued_keys.is_some()
    }

    pub fn has_keys(&self) -> bool {
        self.next_keys.is_some()
    }

    pub fn display_queued_keys(&self, size: usize) -> String {
        if let Some(keys) = &self.queued_keys {
            keys.to_compact_string(size)
        } else {
            "".to_string()
        }
    }

    pub fn display_next_keys(&self, size: usize) -> String {
        if let Some(keys) = &self.next_keys {
            keys.to_compact_string(size)
        } else {
            "".to_string()
        }
    }

    pub fn payee_as_compact(&self, size: usize) -> String {
        self.payee.to_compact_string(self.account_format(), size)
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
        Some(self.points - self.old_points)
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

    pub fn is_proxy_valid(&self) -> bool {
        self.is_proxy_valid
    }
}

impl AccountDisplay for Validator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }

    fn account_format(&self) -> u16 {
        self.account.account_format()
    }
}

impl From<&Validator> for Row<'_> {
    fn from(v: &Validator) -> Self {
        let status = if v.is_proxy_valid() { "[S]" } else { "" };
        let v = v.clone();
        Row::new(vec![
            Text::from(""),
            Text::from(format!("{}/{}", v.runtime(), v.display_name(3),)),
            Text::from(status).alignment(Alignment::Right),
            Text::from(""),
        ])
    }
}
