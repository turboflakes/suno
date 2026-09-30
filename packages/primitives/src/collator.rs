use crate::{
    display::{format_millis, get_elapsed_millis},
    identity::Identity,
    key::AccountKey,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Collator {
    account: NodeAccount,
    status: CollatorStatus,
    last_block_authored: Option<u64>,
    last_block_authored_ts: Option<u128>,
    // Aura slot the last few produced blocks were claimed for, and how many
    // consecutive blocks have been observed authored under that same slot.
    last_slot: Option<u64>,
    blocks_in_slot: u32,
}

impl Collator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
            status: CollatorStatus::default(),
            last_block_authored: None,
            last_block_authored_ts: None,
            last_slot: None,
            blocks_in_slot: 0,
        }
    }

    // Getter methods if needed
    pub fn key(&self) -> &AccountKey {
        self.account.account_key()
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.account.runtime()
    }

    pub fn identity(&self) -> Option<&Identity> {
        self.account.identity().as_ref()
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

    pub fn set_identity(&mut self, identity: Option<Identity>) {
        self.account.set_identity(identity);
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

    pub fn is_invulnerable(&self) -> bool {
        self.status == CollatorStatus::Invulnerable
    }

    pub fn is_waiting(&self) -> bool {
        self.status == CollatorStatus::Waiting
    }

    pub fn last_block_authored(&self) -> Option<u64> {
        self.last_block_authored
    }

    pub fn set_last_block_authored(&mut self, block_number: u64, ts: u128) {
        self.last_block_authored = Some(block_number);
        self.last_block_authored_ts = Some(ts);
    }

    /// Returns how long ago the last produced block was authored, if known.
    pub fn last_block_authored_ago(&self) -> Option<String> {
        let ts = self.last_block_authored_ts?;
        Some(format_millis(get_elapsed_millis(ts), true, false))
    }

    /// Number of consecutive blocks observed authored under the current Aura slot.
    /// With async backing, a single slot claim can cover more than one block, up
    /// to the runtime's `block_processing_velocity`.
    pub fn blocks_in_slot(&self) -> u32 {
        self.blocks_in_slot
    }

    /// Increments the block count for the given Aura slot, either starting a new
    /// count for a new slot or bumping the existing count for the current slot.
    pub fn increment_block_in_slot(&mut self, slot: u64) {
        if self.last_slot == Some(slot) {
            self.blocks_in_slot += 1;
        } else {
            self.last_slot = Some(slot);
            self.blocks_in_slot = 1;
        }
    }

    /// Returns this collator's position in the given Aura authority rotation, if present.
    pub fn slot_index(&self, authorities: &[[u8; 32]]) -> Option<usize> {
        let stash_bytes: [u8; 32] = *self.stash().as_ref();
        authorities.iter().position(|a| *a == stash_bytes)
    }

    /// Returns true if this collator is the Aura author for `current_slot`, per the
    /// round-robin rule `authorities[slot % authorities.len()]`.
    pub fn is_current_slot_author(&self, authorities: &[[u8; 32]], current_slot: u64) -> bool {
        let n = authorities.len() as u64;
        if n == 0 {
            return false;
        }
        self.slot_index(authorities) == Some((current_slot % n) as usize)
    }

    /// Returns the next slot at which this collator is expected to author a block,
    /// based on the Aura round-robin rule `authorities[slot % authorities.len()]`.
    pub fn next_slot(&self, authorities: &[[u8; 32]], current_slot: u64) -> Option<u64> {
        let n = authorities.len() as i64;
        if n == 0 {
            return None;
        }
        let idx = self.slot_index(authorities)? as i64;
        let current_mod = (current_slot % n as u64) as i64;
        let delta = (idx - current_mod).rem_euclid(n);
        let delta = if delta == 0 { n } else { delta };

        Some(current_slot + delta as u64)
    }

    /// Returns a human-readable countdown until `next_slot`, given the constant slot
    /// duration and how long ago `current_slot` was last observed.
    pub fn next_slot_countdown(
        &self,
        authorities: &[[u8; 32]],
        current_slot: u64,
        slot_duration_ms: u64,
        current_slot_ts: u128,
    ) -> Option<String> {
        let next_slot = self.next_slot(authorities, current_slot)?;
        let total_ms = (next_slot - current_slot) * slot_duration_ms;
        let remaining_ms = total_ms.saturating_sub(get_elapsed_millis(current_slot_ts));

        Some(format_millis(remaining_ms, true, false))
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
