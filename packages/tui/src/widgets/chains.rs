use crate::error::TuiError;
use crate::utils::create_substrate_rpc_client_from_url;
use log::{error, info};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, TableState, Widget},
};
use std::sync::{Arc, RwLock};
use subxt::{utils::H256, OnlineClient, SubstrateConfig};
use suno_actions::{network::ConnectionState, Action, ChainAction, SystemAction};
use suno_config::{SupportedRuntime, CONFIG};
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone)]
pub struct ChainsListWidget {
    /// The state is wrapped in an `Arc<RwLock<>>` to allow for shared ownership between the widget and other threads.
    state: Arc<RwLock<ChainsListState>>,
    /// The sender to send actions to update the state to the app.
    tx: UnboundedSender<Action>,
}

#[derive(Debug, Default)]
pub struct ChainsListState {
    chains: Vec<ChainClient>,
    table_state: TableState,
    is_active: bool,
}

#[derive(Debug, Clone)]
pub struct ChainClient {
    runtime: SupportedRuntime,
    client: OnlineClient<SubstrateConfig>,
    state: ConnectionState,
    // last_update value is given in milliseconds
    last_update: u64,
}

impl ChainClient {
    pub fn new(runtime: SupportedRuntime, client: OnlineClient<SubstrateConfig>) -> Self {
        Self {
            runtime,
            client,
            state: ConnectionState::default(),
            last_update: 0,
        }
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

    pub fn update_state(&mut self, state: ConnectionState) {
        self.state = state;
    }

    pub fn block_hash(&self) -> Option<H256> {
        match self.state {
            ConnectionState::Connected(_, hash) => Some(hash),
            _ => None,
        }
    }

    pub async fn validate_genesis(&self) -> Result<(), TuiError> {
        let api = self.client();
        let state_root = self.runtime.chain_state_root_hash();
        let hash = api.genesis_hash();

        if let Some(header) = api.backend().block_header(hash).await? {
            if header.state_root != state_root {
                return Err(TuiError::GenesisError);
            }
        }

        Ok(())
    }

    pub fn is_ready(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Idle | ConnectionState::Connected(_, _)
        )
    }

    pub fn is_offline(&self) -> bool {
        matches!(
            self.state,
            ConnectionState::Reconnecting | ConnectionState::Error(_)
        )
    }
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
                                let mut chain_client = ChainClient::new(chain_name.clone(), client);
                                if let Err(err) = chain_client.validate_genesis().await {
                                    chain_client
                                        .update_state(ConnectionState::Error(err.to_string()));
                                    self.on_err(err.into());
                                }
                                self.on_subscribe(chain_client);
                            }
                            Err(err) => self.on_err(err.into()),
                        }
                    }
                    Err(err) => self.on_err(err),
                }
            }
        }
        // Set the window active.
        self.set_active(true);
    }

    fn on_subscribe(&self, chain_client: ChainClient) {
        let mut state = self.state.write().unwrap();
        state.chains.push(chain_client.clone());
        // Select the first chain.
        if !state.chains.is_empty() {
            state.table_state.select(Some(0));
        }
        if chain_client.is_ready() {
            // Launch a task to subscribe the head of the chain.
            subscribe_finalized_block(chain_client, self.tx.clone());
        }
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        self.tx
            .send(Action::System(SystemAction::Error(err.to_string())))
            .expect("Failed to send error message");
    }

    pub fn move_down(&self) -> Option<ChainClient> {
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
                .map(|i| state.chains[i].clone())
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<ChainClient> {
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
                .map(|i| state.chains[i].clone())
        } else {
            None
        }
    }

    pub fn set_connection_state(&self, runtime: SupportedRuntime, connection: ConnectionState) {
        let mut state = self.state.write().unwrap();
        for chain in state.chains.iter_mut() {
            if chain.runtime == runtime {
                chain.state = connection.clone();
                chain.last_update = 0;
            }
        }
    }

    pub fn tick(&self, value: u64) {
        let mut state = self.state.write().unwrap();
        for chain in state.chains.iter_mut() {
            chain.last_update += value;
        }
    }

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<ChainClient> {
        let state = self.state.read().unwrap();
        state
            .table_state
            .selected()
            .map(|i| state.chains[i].clone())
    }

    pub fn get_chain_client_by_runtime(&self, runtime: &SupportedRuntime) -> Option<ChainClient> {
        let state = self.state.read().unwrap();
        state
            .chains
            .iter()
            .find(|chain| &chain.runtime == runtime)
            .cloned()
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

        let rows = state.chains.iter();
        let widths = [
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Length(12),
        ];

        let table = Table::new(rows, widths)
            .block(block.clone())
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

impl From<&ChainClient> for Row<'_> {
    fn from(cc: &ChainClient) -> Self {
        let cc = cc.clone();
        let ratio = cc.last_update.clamp(0, 6000) as f64 / 6000.0;
        let total_chars = 12;
        let filled_chars = (ratio * total_chars as f64) as usize;

        let progress = format!(
            "{}{}",
            "░".repeat(total_chars - filled_chars),
            "▓".repeat(filled_chars),
        );

        Row::new(vec![
            Text::from(cc.runtime.to_string()),
            Text::from(cc.state.to_string()).alignment(Alignment::Right),
            Text::from(progress.to_string()).alignment(Alignment::Right),
        ])
    }
}

/// Background task that subscribes head block and sends response over channel.
fn subscribe_finalized_block(cc: ChainClient, tx: UnboundedSender<Action>) {
    let api = cc.client.clone();
    let runtime = cc.runtime.clone();
    tokio::spawn(async move {
        match api.blocks().subscribe_finalized().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::Chain(ChainAction::Connection {
                                runtime: runtime.clone(),
                                state: ConnectionState::Connected(
                                    block.number().into(),
                                    block.hash(),
                                ),
                            }));
                        }
                        Err(e) => {
                            // Handle disconnection errors.
                            if e.is_disconnected_will_reconnect() {
                                info!("Lost connection to {} reconnecting...", cc.runtime);
                                let _ = tx.send(Action::Chain(ChainAction::Connection {
                                    runtime: runtime.clone(),
                                    state: ConnectionState::Reconnecting,
                                }));
                                continue;
                            }
                            error!("{}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("error: {:?}", e);
            }
        }
    });
}
