use crate::error::TuiError;
use crate::theme::THEME;
use crate::utils::create_substrate_rpc_client_from_url;
use log::{debug, error, info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style, Styled},
    text::Text,
    widgets::{Block, BorderType, Borders, Cell, Row, StatefulWidget, Table, TableState, Widget},
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::{utils::H256, OnlineClient, SubstrateConfig};
use suno_actions::{network::ConnectionState, Action, ChainAction, SystemAction};
use suno_config::{SupportedRuntime, CONFIG};
use suno_primitives::{
    display::{create_progress_bar_by_millis, format_millis, get_elapsed_millis},
    Epoch, Era, Staking,
};
use tokio::sync::mpsc::UnboundedSender;

type BlockNumber = u64;
type BlockHash = H256;

#[derive(Debug, Clone)]
pub struct Chain {
    runtime: SupportedRuntime,
    client: OnlineClient<SubstrateConfig>,
    best_block: BlockNumber,
    finalized_block: BlockNumber,
    finalized_block_hash: Option<BlockHash>,
    // finalized_block_ts value is the timestamp in milliseconds the finalized block was updated
    finalized_block_ts: u128,
    staking: Option<Staking>,
    era: Option<Era>,
    epoch: Option<Epoch>,
    state: ConnectionState,
}

impl Chain {
    pub fn new(runtime: SupportedRuntime, client: OnlineClient<SubstrateConfig>) -> Self {
        Self {
            runtime,
            client,
            best_block: 0,
            finalized_block: 0,
            finalized_block_ts: 0,
            finalized_block_hash: None,
            staking: None,
            era: None,
            epoch: None,
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

    pub fn block_hash(&self) -> Option<BlockHash> {
        self.finalized_block_hash
    }

    pub async fn validate_genesis(&mut self) -> Result<(), TuiError> {
        let api = self.client();
        let state_root = self.runtime.chain_state_root_hash();
        let hash = api.genesis_hash();

        if let Some(header) = api.backend().block_header(hash).await? {
            if header.state_root != state_root {
                let err = TuiError::GenesisError;
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

    pub fn spawn_fetch_epoch_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        chain_key: &ChainKey,
    ) {
        let api = api.clone();
        let chain_key = chain_key.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(e) =
                fetch_and_send_chain_data(&api, block_hash, &chain_key, tx.clone()).await
            {
                let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
            }
        });
    }
}

impl Widget for &ChainsListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let (table_style, highlight_style) = match state.is_active {
            true => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::Black).bg(Color::White),
            ),
            false => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Blue),
            ),
        };

        let block = Block::new()
            // .title("Chains")
            // .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::TOP | Borders::LEFT | Borders::BOTTOM)
            .border_type(BorderType::Plain);

        let rows = state.chains_iter();
        let widths = [
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(4),
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .header(
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(Text::from("best").alignment(Alignment::Right)),
                    Cell::from(Text::from("finalized").alignment(Alignment::Right)),
                    Cell::from(""),
                    Cell::from(""),
                ])
                .set_style(THEME.table.header),
            )
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

impl From<&Chain> for Row<'_> {
    fn from(chain: &Chain) -> Self {
        let elapsed = get_elapsed_millis(chain.finalized_block_ts);
        let progress = create_progress_bar_by_millis(elapsed, 6);

        Row::new(vec![
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
        ])
    }
}

// Helper functions
//
/// Background task that subscribes head block and sends response over channel.
fn subscribe_best_block(chain: &Chain, tx: UnboundedSender<Action>) {
    let api = chain.client.clone();
    let runtime = chain.runtime.clone();
    tokio::spawn(async move {
        match api.blocks().subscribe_best().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::Chain(ChainAction::UpdateBestBlock(
                                runtime.clone(),
                                block.number().into(),
                            )));
                        }
                        Err(e) => {
                            if e.is_disconnected_will_reconnect() {
                                warn!("Lost connection to {} reconnecting...", runtime.clone());
                                let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                                    runtime.clone(),
                                    ConnectionState::Reconnecting,
                                )));
                                continue;
                            }
                            error!("subscribe_best result error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("subscribe_best error: {:?}", e);
            }
        }
    });
}

/// Background task that subscribes finalized block and sends response over channel.
fn subscribe_finalized_block(chain: &Chain, tx: UnboundedSender<Action>) {
    let api = chain.client.clone();
    let runtime = chain.runtime.clone();
    tokio::spawn(async move {
        match api.blocks().subscribe_finalized().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::Chain(ChainAction::UpdateFinalizedBlock(
                                runtime.clone(),
                                block.number().into(),
                                block.hash(),
                            )));

                            // Everytime a new block is received, update the connection state to connected.
                            // Used as KEEPALIVE in case of reconnections and initialization
                            let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                                runtime.clone(),
                                ConnectionState::Connected,
                            )));
                        }
                        Err(e) => {
                            if e.is_disconnected_will_reconnect() {
                                info!("Lost connection to {} reconnecting...", runtime.clone());
                                let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                                    runtime.clone(),
                                    ConnectionState::Reconnecting,
                                )));
                                continue;
                            }
                            error!("subscribe_finalized result error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("subscribe_finalized error: {:?}", e);
            }
        }
    });
}

async fn fetch_and_send_chain_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    tx: UnboundedSender<Action>,
) -> Result<(), TuiError> {
    let (epoch_result,) = match runtime {
        SupportedRuntime::Polkadot => {
            // TODO: Add more fetches here to run them in parallel/
            tokio::join!(suno_polkadot::fetch_epoch_data(api, block_hash),)
        }
        SupportedRuntime::Kusama => {
            tokio::join!(suno_kusama::fetch_epoch_data(api, block_hash),)
        }
        SupportedRuntime::Paseo => {
            tokio::join!(suno_paseo::fetch_epoch_data(api, block_hash),)
        }
        SupportedRuntime::Westend => {
            tokio::join!(suno_westend::fetch_epoch_data(api, block_hash),)
        }
        _ => {
            error!("Unsupported runtime: {:?}", runtime);
            return Ok(());
        }
    };

    // Handle epoch result
    match epoch_result {
        Ok(epoch) => {
            tx.send(Action::Chain(ChainAction::UpdateEpoch(
                runtime.clone(),
                epoch,
            )))?;
        }
        Err(e) => warn!(
            "Failed to fetch epoch data for {:?}: {}",
            runtime.to_string(),
            e
        ),
    }

    Ok(())
}
