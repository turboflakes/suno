use crate::{
    identity::Identity,
    node_account::{AccountDisplay, NodeAccount},
};
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CollatorStatus {
    /// Collator is an authority in the current Aura slot set, displayed as [A]
    Authority,
    /// Collator is part of the fixed invulnerable set, displayed as [I]
    Invulnerable,
    /// Collator is a registered candidate waiting to become an authority, displayed as [W]
    Waiting,
    /// Collator status is unknown or not yet determined, displayed as [U]
    #[default]
    Unknown,
}

impl std::fmt::Display for CollatorStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Authority => write!(f, "[A]"),
            Self::Invulnerable => write!(f, "[I]"),
            Self::Waiting => write!(f, "[W]"),
            Self::Unknown => write!(f, "[U]"),
        }
    }
}

/// Specific types using composition
#[derive(Debug, Clone)]
pub struct Collator {
    account: NodeAccount,
    status: CollatorStatus,
}

impl Collator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
            status: CollatorStatus::default(),
        }
    }

    // Getter methods if needed
    pub fn runtime(&self) -> SupportedRuntime {
        self.account.runtime()
    }

    pub fn identity(&self) -> Option<&Identity> {
        self.account.identity().as_ref()
    }

    pub fn status(&self) -> &CollatorStatus {
        &self.status
    }

    pub fn set_status(&mut self, status: CollatorStatus) {
        self.status = status;
    }

    pub fn is_authority(&self) -> bool {
        self.status == CollatorStatus::Authority
    }

    pub fn display_name(&self, size: usize) -> String {
        if let Some(identity) = self.identity() {
            format!("{} ({})", identity, self.to_compact_string(size))
        } else {
            self.to_compact_string(size)
        }
    }
}

impl AccountDisplay for Collator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }

    fn account_format(&self) -> u16 {
        self.account.account_format()
    }
}
