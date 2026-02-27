use crate::bridge::subscribe::{subscribe_best_block, subscribe_finalized_block};
use crate::theme::THEME;
use crate::utils::create_substrate_rpc_client_from_url;
use crate::widgets::scrollbar::render_scrollbar;
use log::debug;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::Styled,
    text::Text,
    widgets::{Block, Cell, Padding, Row, StatefulWidget, Table, TableState, Widget},
};
use sp_arithmetic::Permill;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::{utils::H256, OnlineClient, SubstrateConfig};
use suno_actions::{network::ConnectionState, Action, SystemAction};
use suno_config::{SupportedRuntime, CONFIG};
use suno_error::Error;
use suno_primitives::{
    display::{create_progress_bar_by_millis, format_millis, get_elapsed_millis},
    Epoch, Era,
};
use tokio::sync::mpsc::UnboundedSender;

type BlockNumber = u64;
type BlockHash = H256;

#[derive(Debug, Clone)]
pub struct Chain {
    // Chain runtime details
    runtime: SupportedRuntime,
    // Api client details
    client: OnlineClient<SubstrateConfig>,
    // Best block number
    best_block: BlockNumber,
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
    pub fn new(runtime: SupportedRuntime, client: OnlineClient<SubstrateConfig>) -> Self {
        Self {
            runtime,
            client,
            best_block: 0,
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

    pub fn key(&self) -> &SupportedRuntime {
        &self.runtime
    }

    pub fn name(&self) -> &str {
        &self.runtime.as_str()
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.runtime
    }

    pub fn client(&self) -> &OnlineClient<SubstrateConfig> {
        &self.client
    }

    pub fn state(&self) -> &ConnectionState {
        &self.state
    }

    pub fn finalized_block(&self) -> u64 {
        self.finalized_block
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

    pub async fn validate_genesis(&mut self) -> Result<(), Error> {
        let api = self.client();
        let state_root = self.runtime.chain_state_root_hash();
        let hash = api.genesis_hash();

        if let Some(header) = api.backend().block_header(hash).await? {
            if header.state_root != state_root {
                let err = Error::GenesisError;
                self.set_state(ConnectionState::Error(err.to_string()));
                return Err(err);
            }
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
            ConnectionState::Idle | ConnectionState::Reconnecting | ConnectionState::Error(_)
        )
    }

    pub fn set_state(&mut self, state: ConnectionState) {
        self.state = state;
    }
}

type ChainKey = SupportedRuntime;

#[derive(Debug, Default)]
pub struct ChainsListState {
    chains: HashMap<ChainKey, Chain>,
    chains_order: Vec<ChainKey>,
    table_state: TableState,
    is_active: bool,
}

impl ChainsListState {
    pub fn add_chain(&mut self, chain: Chain) {
        let key = chain.key();
        if !self.chains.contains_key(&key) {
            self.chains_order.push(key.clone());
        }
        self.chains.insert(key.clone(), chain);
    }

    pub fn set_best_block(&mut self, chain_key: &ChainKey, block_number: BlockNumber) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            if chain.best_block != block_number {
                chain.best_block = block_number;
                return true;
            }
        }
        false
    }

    pub fn set_finalized_block(
        &mut self,
        chain_key: &ChainKey,
        block_number: BlockNumber,
        block_hash: BlockHash,
    ) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            if chain.finalized_block != block_number {
                chain.finalized_block = block_number;
                chain.finalized_block_hash = Some(block_hash);
                chain.finalized_block_ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                return true;
            }
        }
        false
    }

    pub fn set_connection_state(&mut self, chain_key: &ChainKey, state: ConnectionState) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            if chain.state != state {
                chain.state = state;
                return true;
            }
        }
        false
    }

    pub fn set_era(&mut self, chain_key: &ChainKey, data: Era) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.era = Some(data);
            return true;
        }
        false
    }

    pub fn set_epoch(&mut self, chain_key: &ChainKey, data: Epoch) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.epoch = Some(data);
            return true;
        }
        false
    }

    pub fn set_active_vals(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.active_vals = counter;
            return true;
        }
        false
    }

    pub fn set_total_vals(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.total_vals = counter;
            return true;
        }
        false
    }

    pub fn set_active_noms(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.active_noms = counter;
            return true;
        }
        false
    }

    pub fn set_total_noms(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.total_noms = counter;
            return true;
        }
        false
    }

    pub fn set_total_staked(&mut self, chain_key: &ChainKey, value: Permill) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.total_staked_pm = value;
            return true;
        }
        false
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn _get_chain_by_key(&self, chain_key: &ChainKey) -> Option<&Chain> {
        self.chains.get(chain_key)
    }

    pub fn get_chain_by_key_cloned(&self, chain_key: &ChainKey) -> Option<Chain> {
        self.chains.get(chain_key).cloned()
    }

    // Helper method to get chain by table index
    pub fn get_chain_by_index(&self, index: usize) -> Option<&Chain> {
        self.chains_order
            .get(index)
            .and_then(|key| self.chains.get(key))
    }

    pub fn get_chain_by_index_cloned(&self, index: usize) -> Option<Chain> {
        self.get_chain_by_index(index).cloned()
    }

    /// Returns an iterator of chains in display order
    pub fn chains_iter(&self) -> impl Iterator<Item = &Chain> {
        self.chains_order
            .iter()
            .filter_map(move |key| self.chains.get(key))
    }

    pub fn get_selected_ref(&self) -> Option<&Chain> {
        self.table_state
            .selected()
            .and_then(|i| self.get_chain_by_index(i))
    }
}

#[derive(Debug, Clone)]
pub struct ChainsListWidget {
    /// The state is wrapped in an `Arc<RwLock<>>` to allow for shared ownership between the widget and other threads.
    state: Arc<RwLock<ChainsListState>>,
    /// The sender to send actions to update the state to the app.
    tx: UnboundedSender<Action>,
}

impl ChainsListWidget {
    pub fn new(tx: UnboundedSender<Action>) -> Self {
        Self {
            state: Arc::new(RwLock::new(ChainsListState::default())),
            tx,
        }
    }
    /// Initialize OnlineClients for each configured chain.
    pub async fn on_init(&self) {
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                match create_substrate_rpc_client_from_url(&chain_config.rpc_url).await {
                    Ok(rpc_client) => {
                        match OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client).await {
                            Ok(client) => {
                                let mut chain = Chain::new(chain_name.clone(), client);
                                if let Err(err) = chain.validate_genesis().await {
                                    self.error(err.into());
                                }
                                self.add_chain(&chain);
                                self.subscribe(&chain);
                            }
                            Err(err) => self.error(err.into()),
                        }
                    }
                    Err(err) => self.error(err),
                }
            }
        }
        self.init_table();
    }

    fn add_chain(&self, chain: &Chain) {
        let mut state = self.state.write().unwrap();
        state.add_chain(chain.clone());
    }

    fn subscribe(&self, chain: &Chain) {
        if chain.is_validated() {
            subscribe_best_block(chain, self.tx.clone());
            subscribe_finalized_block(chain, self.tx.clone());
        }
    }

    fn error(&self, err: Box<dyn std::error::Error>) {
        self.tx
            .send(Action::System(SystemAction::Error(err.to_string())))
            .expect("Failed to send error message");
    }

    pub fn move_down(&self) -> Option<Chain> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.chains.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.scroll_down_by(1);
            }
            state
                .table_state
                .selected()
                .and_then(|i| state.get_chain_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Chain> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = state.chains.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.scroll_up_by(1);
            }
            state
                .table_state
                .selected()
                .and_then(|i| state.get_chain_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn init_table(&self) {
        let mut state = self.state.write().unwrap();
        if !state.chains.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    pub fn is_active(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_active()
    }

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Chain> {
        let state = self.state.read().unwrap();
        state
            .table_state
            .selected()
            .and_then(|i| state.get_chain_by_index_cloned(i))
    }

    pub fn get_chain_by_runtime(&self, runtime: &SupportedRuntime) -> Option<Chain> {
        let state = self.state.read().unwrap();
        state.get_chain_by_key_cloned(runtime)
    }

    pub fn get_api_and_block_hash(
        &self,
        runtime: &SupportedRuntime,
    ) -> Option<(OnlineClient<SubstrateConfig>, H256)> {
        let chain = self.get_chain_by_runtime(runtime)?;

        if !chain.is_connected() {
            debug!("Chain {} not connected", runtime);
            return None;
        }

        let block_hash = chain.block_hash()?;
        Some((chain.client().clone(), block_hash))
    }

    pub fn update_connection_state(
        &self,
        chain_key: &ChainKey,
        connection_state: ConnectionState,
    ) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_connection_state(chain_key, connection_state)
    }

    pub fn update_best_block(&self, chain_key: &ChainKey, block_number: BlockNumber) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_best_block(chain_key, block_number)
    }

    pub fn update_finalized_block(
        &self,
        chain_key: &ChainKey,
        block_number: BlockNumber,
        block_hash: BlockHash,
    ) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_finalized_block(chain_key, block_number, block_hash)
    }

    pub fn update_era(&self, chain_key: &ChainKey, era: Era) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_era(chain_key, era)
    }

    pub fn update_epoch(&self, chain_key: &ChainKey, epoch: Epoch) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_epoch(chain_key, epoch)
    }

    pub fn update_active_validators(&self, chain_key: &ChainKey, count: u32) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_active_vals(chain_key, count)
    }

    pub fn update_total_validators(&self, chain_key: &ChainKey, count: u32) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_total_vals(chain_key, count)
    }

    pub fn update_active_nominators(&self, chain_key: &ChainKey, count: u32) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_active_noms(chain_key, count)
    }

    pub fn update_total_nominators(&self, chain_key: &ChainKey, count: u32) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_total_noms(chain_key, count)
    }

    pub fn update_total_staked(&self, chain_key: &ChainKey, value: Permill) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_total_staked(chain_key, value)
    }
}

impl Widget for &ChainsListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let block = Block::new()
            .set_style(THEME.block.menu_top(state.is_active))
            .padding(Padding::symmetric(0, 1));

        let rows = state.chains_iter();
        let widths = [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(4),
            Constraint::Length(1),
        ];

        let header_cells = vec![
            Cell::from(""),
            Cell::from(Text::from("chains").alignment(Alignment::Left)),
            Cell::from(Text::from("best").alignment(Alignment::Right)),
            Cell::from(Text::from("finalized").alignment(Alignment::Right)),
            Cell::from(""),
            Cell::from(""),
            Cell::from(""),
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(header_cells).set_style(THEME.table.header(state.is_active)))
            .style(THEME.table.base)
            .row_highlight_style(THEME.table.row_highlight(state.is_active))
            .highlight_symbol(THEME.table.highlight_symbol(state.is_active));

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        // Render scrollbar when active
        if state.is_active && state.chains.len() >= area.height.saturating_sub(2) as usize {
            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
                ..area
            };
            if let Some(row_index) = state.table_state.selected() {
                render_scrollbar(row_index, state.chains.len(), scrollbar_area, buf);
            }
        }
    }
}

impl From<&Chain> for Row<'_> {
    fn from(chain: &Chain) -> Self {
        let elapsed = get_elapsed_millis(chain.finalized_block_ts);
        let progress = create_progress_bar_by_millis(elapsed, 6);

        Row::new(vec![
            Text::from(""),
            Text::from(format!(
                "{}{}",
                chain.state.to_string(),
                chain.runtime.to_string()
            )),
            Text::from(format!("#{}", chain.best_block.to_string())).alignment(Alignment::Right),
            Text::from(format!("#{}", chain.finalized_block.to_string()))
                .alignment(Alignment::Right),
            Text::from(progress.to_string()).alignment(Alignment::Right),
            Text::from(format_millis(elapsed)).alignment(Alignment::Right),
            Text::from(""),
        ])
    }
}
