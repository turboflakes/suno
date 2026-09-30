use crate::display::{format_millis, get_elapsed_millis};
use subxt::{config::substrate::DigestItem, ext::codec::Decode};

/// Consensus engine id used by `pallet_aura` to tag its slot-claim pre-runtime digest.
const AURA_ENGINE_ID: [u8; 4] = *b"aura";

/// Extracts the Aura slot a block was authored for, from its pre-runtime digest.
pub fn extract_aura_slot(digest_logs: &[DigestItem]) -> Option<u64> {
    digest_logs.iter().find_map(|item| match item {
        DigestItem::PreRuntime(id, data) if *id == AURA_ENGINE_ID => {
            u64::decode(&mut &data[..]).ok()
        }
        _ => None,
    })
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Aura {
    // Current session index
    current_session_index: Option<u32>,
    // Current Aura slot
    current_slot: Option<u64>,
    // Timestamp in milliseconds of the last current_slot update
    current_slot_ts: u128,
    // Current Aura authority set, in on-chain rotation order
    authorities: Vec<[u8; 32]>,
    // Fixed invulnerable collator set, from `CollatorSelection::Invulnerables`
    invulnerables: Vec<[u8; 32]>,
    // Aura slot duration in milliseconds, constant for the runtime
    slot_duration_ms: Option<u64>,
    // Expected average block creation in milliseconds, constant
    // calculated based on the relay chain expected block time
    // and the chain's block_processing_velocity set at runtime
    block_time_ms: Option<u64>,
}

impl Aura {
    pub fn current_session_index(&self) -> Option<u32> {
        self.current_session_index
    }

    pub fn set_current_session_index(&mut self, index: Option<u32>) {
        self.current_session_index = index;
    }

    pub fn current_slot(&self) -> Option<u64> {
        self.current_slot
    }

    pub fn current_slot_ts(&self) -> u128 {
        self.current_slot_ts
    }

    pub fn slot_duration_ms(&self) -> Option<u64> {
        self.slot_duration_ms
    }

    pub fn authorities(&self) -> &[[u8; 32]] {
        &self.authorities
    }

    pub fn block_time_ms(&self) -> Option<u64> {
        self.block_time_ms
    }

    pub fn set_current_slot(&mut self, current_slot: Option<u64>) {
        self.current_slot = current_slot;
    }

    pub fn set_current_slot_ts(&mut self, ts: u128) {
        self.current_slot_ts = ts;
    }

    pub fn set_slot_duration_ms(&mut self, slot_duration_ms: Option<u64>) {
        self.slot_duration_ms = slot_duration_ms;
    }

    pub fn invulnerables(&self) -> &[[u8; 32]] {
        &self.invulnerables
    }

    /// Current authorities that are not part of the invulnerable set, i.e. the
    /// collators that got in through the candidacy (permissionless) route.
    pub fn permissionless(&self) -> Vec<[u8; 32]> {
        self.authorities
            .iter()
            .filter(|a| !self.invulnerables.contains(a))
            .copied()
            .collect()
    }

    pub fn set_invulnerables(&mut self, invulnerables: Vec<[u8; 32]>) {
        self.invulnerables = invulnerables;
    }

    pub fn set_authorities(&mut self, authorities: Vec<[u8; 32]>) {
        self.authorities = authorities;
    }

    pub fn set_block_time_ms(&mut self, block_time_ms: Option<u64>) {
        self.block_time_ms = block_time_ms;
    }

    /// Blocks elapsed in the current session, assuming `pallet_session::PeriodicSessions`
    /// with `Offset = 0`, as configured on the supported system parachains.
    fn session_blocks_elapsed(&self, current_block_number: u64, duration_bn: u64) -> u64 {
        current_block_number % duration_bn
    }

    /// Number of blocks expected per slot, based on the slot duration and block time.
    pub fn number_blocks_expected(&self) -> Option<u64> {
        self.slot_duration_ms()?.checked_div(self.block_time_ms()?)
    }

    /// Progress (0.0-1.0) through the current session (parachains only).
    pub fn session_progress(&self, current_block_number: u64, duration_bn: u64) -> f64 {
        self.session_blocks_elapsed(current_block_number, duration_bn) as f64 / duration_bn as f64
    }

    /// Block number at which the next session is expected to start.
    pub fn next_session_expected_block(&self, current_block_number: u64, duration_bn: u64) -> u64 {
        current_block_number - self.session_blocks_elapsed(current_block_number, duration_bn)
            + duration_bn
    }

    /// Human-readable time until the current session ends, using the Aura slot duration
    /// as the expected time per block.
    pub fn session_countdown_time(&self, current_block_number: u64, duration_bn: u64) -> String {
        let remaining_blocks =
            duration_bn - self.session_blocks_elapsed(current_block_number, duration_bn);
        let block_time_ms = self.block_time_ms().unwrap_or(0);

        format_millis(remaining_blocks * block_time_ms, true, false)
    }

    /// Progress (0.0-1.0) through the current Aura slot, based on how long ago
    /// `current_slot` was last observed relative to `slot_duration_ms`.
    pub fn slot_progress(&self) -> f64 {
        let Some(duration_ms) = self.slot_duration_ms else {
            return 0.0;
        };
        if duration_ms == 0 {
            return 0.0;
        }
        let elapsed_ms = get_elapsed_millis(self.current_slot_ts);
        (elapsed_ms as f64 / duration_ms as f64).min(1.0)
    }

    /// Human-readable countdown until the current slot elapses.
    pub fn slot_countdown_time(&self) -> String {
        let Some(duration_ms) = self.slot_duration_ms else {
            return format_millis(0, true, false);
        };
        let elapsed_ms = get_elapsed_millis(self.current_slot_ts);
        let remaining_ms = duration_ms.saturating_sub(elapsed_ms);

        format_millis(remaining_ms, true, false)
    }
}
