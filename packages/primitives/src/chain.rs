use crate::network::ConnectionState;
use crate::{Epoch, Era};
use sp_arithmetic::Permill;
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
    // RPC Connection status
    state: ConnectionState,
}

impl Chain {
    pub fn new(runtime: SupportedRuntime, client: OnlineClient<CustomConfig>) -> Self {
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
            state: ConnectionState::default(),
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
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalidg genesis hash for {0}")]
    InvalidGenesisHash(String),
}
