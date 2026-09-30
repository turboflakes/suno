use crate::display::{format_millis, get_elapsed_millis};
use crate::network::ConnectionState;
use crate::{Aura, Epoch, Era};
use sp_arithmetic::Permill;
use std::collections::VecDeque;
use subxt::{utils::H256, OnlineClient};
use suno_config::{CustomConfig, SupportedRuntime};

pub type BlockNumber = u64;
pub type BlockHash = H256;

#[derive(Debug, Clone)]
pub struct Chain {
    // Chain runtime details
    runtime: SupportedRuntime,
    // Api client details
    client: OnlineClient<CustomConfig>,
    // Best block number
    best_block: BlockNumber,
    // Best block timestamp in milliseconds
    best_block_ts: u128,
    // Finalized block number
    finalized_block: BlockNumber,
    // Finalized block timestamp
    finalized_block_hash: Option<BlockHash>,
    // Finalized block timestamp in milliseconds
    finalized_block_ts: u128,
    // Era details
    era: Option<Era>,
    // Epoch details
    epoch: Option<Epoch>,
    // Active validators
    active_vals: u32,
    // Total validators
    total_vals: u32,
    // Active nominators
    active_noms: u32,
    // Total nominators
    total_noms: u32,
    // Total staked rate
    total_staked_pm: Permill,
    // Aura consensus details (parachains only)
    aura: Option<Aura>,
    // RPC Connection status
    state: ConnectionState,
    // Current consensus slot (Babe for relay chains, Aura for parachains)
    current_slot: Option<u64>,
    // Timestamp in milliseconds of the last current_slot update
    current_slot_ts: u128,
    // Most recent `(block number, slot)` pairs observed, oldest first
    recent_blocks: RecentBlocks,
}

impl Chain {
    pub fn new(runtime: SupportedRuntime, client: OnlineClient<CustomConfig>) -> Self {
        let aura = if !runtime.is_relay_chain() {
            Some(Aura::default())
        } else {
            None
        };
        Self {
            runtime,
            client,
            best_block: 0,
            best_block_ts: 0,
            finalized_block: 0,
            finalized_block_hash: None,
            finalized_block_ts: 0,
            era: None,
            epoch: None,
            active_vals: 0,
            total_vals: 0,
            active_noms: 0,
            total_noms: 0,
            total_staked_pm: Permill::zero(),
            aura,
            state: ConnectionState::default(),
            recent_blocks: RecentBlocks::default(),
            current_slot: None,
            current_slot_ts: 0,
        }
    }

    pub fn key(&self) -> SupportedRuntime {
        self.runtime
    }

    pub fn name(&self) -> &str {
        self.runtime.as_str()
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.runtime
    }

    pub fn client(&self) -> &OnlineClient<CustomConfig> {
        &self.client
    }

    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn best_block(&self) -> u64 {
        self.best_block
    }

    pub fn best_block_ts(&self) -> u128 {
        self.best_block_ts
    }

    pub fn finalized_block(&self) -> u64 {
        self.finalized_block
    }

    pub fn finalized_block_hash(&self) -> &Option<BlockHash> {
        &self.finalized_block_hash
    }

    pub fn finalized_block_ts(&self) -> u128 {
        self.finalized_block_ts
    }

    pub fn era(&self) -> &Option<Era> {
        &self.era
    }

    pub fn epoch(&self) -> &Option<Epoch> {
        &self.epoch
    }

    pub fn aura(&self) -> &Option<Aura> {
        &self.aura
    }

    pub fn get_mut_aura(&mut self) -> Option<&mut Aura> {
        self.aura.as_mut()
    }

    pub fn active_validators_count(&self) -> u32 {
        self.active_vals
    }

    pub fn total_validators_count(&self) -> u32 {
        self.total_vals
    }

    pub fn waiting_validators_count(&self) -> u32 {
        self.total_vals.saturating_sub(self.active_vals)
    }

    pub fn active_nominators_count(&self) -> u32 {
        self.active_noms
    }

    pub fn total_nominators_count(&self) -> u32 {
        self.total_noms
    }

    pub fn waiting_nominators_count(&self) -> u32 {
        self.total_noms.saturating_sub(self.active_noms)
    }

    pub fn total_staked_percentage(&self) -> String {
        let percentage = self.total_staked_pm.deconstruct() as f64 / 10_000.0;
        format!("{:.1}%", percentage)
    }

    pub fn block_hash(&self) -> Option<BlockHash> {
        self.finalized_block_hash
    }

    pub fn validate_genesis(&mut self) -> Result<(), Error> {
        if self.client().genesis_hash() != self.runtime.chain_genesis_hash() {
            let err = Error::InvalidGenesisHash(self.runtime.to_string());
            self.set_state(ConnectionState::Error(err.to_string()));
            return Err(err);
        }

        self.set_state(ConnectionState::Validated);

        Ok(())
    }

    pub fn is_validated(&self) -> bool {
        matches!(self.state, ConnectionState::Validated)
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    pub fn is_offline(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Idle | ConnectionState::Offline | ConnectionState::Error(_)
        )
    }

    /// Average time between blocks in milliseconds, over the recorded samples.
    pub fn average_block_time_ms(&self, duration_ms: u64) -> Option<u64> {
        self.recent_blocks.average_block_time_ms(duration_ms)
    }

    /// Records the `slot` a block was authored in. Blocks must be recorded in order;
    /// repeated or older blocks are ignored.
    pub fn add_recent_block(&mut self, block_number: u64, slot: u64) {
        self.recent_blocks.record(block_number, slot);
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    pub fn set_best_block(&mut self, block_number: BlockNumber) {
        self.best_block = block_number;
    }

    pub fn set_best_block_ts(&mut self, ts: u128) {
        self.best_block_ts = ts;
    }

    pub fn set_finalized_block(&mut self, block_number: BlockNumber) {
        self.finalized_block = block_number;
    }

    pub fn set_finalized_block_hash(&mut self, block_hash: Option<BlockHash>) {
        self.finalized_block_hash = block_hash;
    }

    pub fn set_finalized_block_ts(&mut self, ts: u128) {
        self.finalized_block_ts = ts;
    }

    pub fn set_era(&mut self, era: Option<Era>) {
        self.era = era;
    }

    pub fn set_epoch(&mut self, epoch: Option<Epoch>) {
        self.epoch = epoch;
    }

    pub fn set_active_vals(&mut self, counter: u32) {
        self.active_vals = counter;
    }

    pub fn set_total_vals(&mut self, counter: u32) {
        self.total_vals = counter;
    }

    pub fn set_active_noms(&mut self, counter: u32) {
        self.active_noms = counter;
    }

    pub fn set_total_noms(&mut self, counter: u32) {
        self.total_noms = counter;
    }

    pub fn set_total_staked_pm(&mut self, value: Permill) {
        self.total_staked_pm = value;
    }

    pub fn current_slot(&self) -> Option<u64> {
        self.current_slot
    }

    pub fn current_slot_ts(&self) -> u128 {
        self.current_slot_ts
    }

    pub fn set_current_slot(&mut self, current_slot: Option<u64>) {
        self.current_slot = current_slot;
    }

    pub fn set_current_slot_ts(&mut self, ts: u128) {
        self.current_slot_ts = ts;
    }

    /// Progress (0.0-1.0) through the current slot, based on how long ago
    /// `current_slot` was last observed relative to `slot_duration_ms`.
    pub fn slot_progress(&self, slot_duration_ms: u64) -> f64 {
        if slot_duration_ms == 0 {
            return 0.0;
        }
        let elapsed_ms = get_elapsed_millis(self.current_slot_ts);
        (elapsed_ms as f64 / slot_duration_ms as f64).min(1.0)
    }

    /// Human-readable countdown until the current slot elapses.
    pub fn slot_countdown_time(&self, slot_duration_ms: u64) -> String {
        let elapsed_ms = get_elapsed_millis(self.current_slot_ts);
        let remaining_ms = slot_duration_ms.saturating_sub(elapsed_ms);

        format_millis(remaining_ms, true, false)
    }

    pub fn set_slot_duration_ms(&mut self, slot_duration_ms: Option<u64>) {
        self.aura
            .get_or_insert_with(Aura::default)
            .set_slot_duration_ms(slot_duration_ms);
    }

    pub fn set_aura_block_time_ms(&mut self, block_time_ms: Option<u64>) {
        self.aura
            .get_or_insert_with(Aura::default)
            .set_block_time_ms(block_time_ms);
    }

    pub fn set_aura_invulnerables(&mut self, invulnerables: Vec<[u8; 32]>) {
        self.aura
            .get_or_insert_with(Aura::default)
            .set_invulnerables(invulnerables);
    }

    pub fn set_aura_authorities(&mut self, aura_authorities: Vec<[u8; 32]>) {
        self.aura
            .get_or_insert_with(Aura::default)
            .set_authorities(aura_authorities);
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalidg genesis hash for {0}")]
    InvalidGenesisHash(String),
}

/// Number of recent `(block number, slot)` samples kept to average the block time over.
const BLOCK_SAMPLES: usize = 50;

/// Rolling window of `(block number, slot)` samples used to estimate average block time,
/// generic over any slot-based consensus engine (Aura, Babe, ...).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecentBlocks {
    blocks: VecDeque<(u64, u64)>,
}

impl RecentBlocks {
    /// Records the slot a block was authored in. Blocks must be recorded in order;
    /// repeated or older blocks are ignored.
    pub fn record(&mut self, block_number: u64, slot: u64) {
        if self
            .blocks
            .back()
            .is_some_and(|(last_block, _)| block_number <= *last_block)
        {
            return;
        }
        if self.blocks.len() == BLOCK_SAMPLES {
            self.blocks.pop_front();
        }
        self.blocks.push_back((block_number, slot));
    }

    /// Average time between blocks in milliseconds, given the fixed slot duration.
    ///
    /// Each slot lasts `slot_duration_ms`, so the slots spanned by the blocks give the
    /// elapsed time, and slots where no block was authored count as elapsed time too.
    pub fn average_block_time_ms(&self, slot_duration_ms: u64) -> Option<u64> {
        let (first_block, first_slot) = *self.blocks.front()?;
        let (last_block, last_slot) = *self.blocks.back()?;
        let blocks = last_block.checked_sub(first_block).filter(|b| *b > 0)?;
        let slots = last_slot.checked_sub(first_slot)?;

        Some(slots * slot_duration_ms / blocks)
    }
}
