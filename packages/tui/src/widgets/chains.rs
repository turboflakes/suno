use crate::app::Action;
use crate::config::{SupportedRuntime, CONFIG};
use crate::utils::create_substrate_rpc_client_from_url;
use crate::widgets::scrollbar::render_scrollbar;
use log::{error, info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Clear, Row, StatefulWidget, Table, TableState, Widget},
};
use std::sync::{Arc, RwLock};
use subxt::{OnlineClient, SubstrateConfig};
use tokio::sync::mpsc::UnboundedSender;

pub type BlockNumber = u32;

#[derive(Debug, Clone, Default)]
pub struct ChainsListWidget {
    state: Arc<RwLock<ChainsListState>>,
}

#[derive(Debug, Default)]
pub struct ChainsListState {
    chains: Vec<ChainClient>,
    table_state: TableState,
    is_active: bool,
}

#[derive(Debug, Clone)]
pub struct ChainClient {
    pub runtime: SupportedRuntime,
    pub client: OnlineClient<SubstrateConfig>,
    state: ConnectionState,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ConnectionState {
    #[default]
    Idle,
    Connecting,
    Connected(BlockNumber),
    Error(String),
}

impl std::fmt::Display for ConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "-"),
            Self::Connecting => write!(f, "↺"),
            Self::Connected(block_number) => write!(f, "#{}", block_number),
            Self::Error(_) => write!(f, "✗"),
        }
    }
}

impl ChainsListWidget {
    /// Initialize OnlineClients for each configured chain.
    pub async fn on_init(&self, tx: &UnboundedSender<Action>) {
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                info!("Chain: {}", chain_name);
                match create_substrate_rpc_client_from_url(&chain_config.rpc_url).await {
                    Ok(rpc_client) => {
                        match OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client).await {
                            Ok(client) => {
                                let chain_client = ChainClient {
                                    runtime: chain_name.clone(),
                                    client,
                                    state: ConnectionState::Connecting,
                                };
                                self.on_connecting(chain_client, tx.clone())
                            }
                            Err(err) => self.on_err(Box::new(err)),
                        }
                    }
                    Err(err) => self.on_err(err),
                }
            }
        }
        // Set the window active.
        self.set_active(true);
    }

    fn on_connecting(&self, chain_client: ChainClient, tx: UnboundedSender<Action>) {
        let mut state = self.state.write().unwrap();
        state.chains.push(chain_client.clone());
        // Select the first chain.
        if !state.chains.is_empty() {
            state.table_state.select(Some(0));
        }
        // Launch a task to subscribe the head of the chain.
        subscribe_best_block(chain_client, tx.clone());
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
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
                chain.state = connection.clone()
            }
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
            .title(" Chains ")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let rows = state.chains.iter();
        let widths = [Constraint::Fill(1), Constraint::Length(10)];
        let table = Table::new(rows, widths)
            .block(block)
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        if state.is_active {
            // Render scrollbar.
            let scrollbar_area = Rect {
                y: area.y + 1,
                height: area.height - 2,
                ..area
            };
            let row_index = state.table_state.selected().unwrap();
            render_scrollbar(row_index, state.chains.len(), scrollbar_area, buf);
        }
    }
}

impl From<&ChainClient> for Row<'_> {
    fn from(cc: &ChainClient) -> Self {
        let cc = cc.clone();
        Row::new(vec![cc.runtime.to_string(), cc.state.to_string()])
    }
}

/// Background task that subscribes head block and sends response over channel.
fn subscribe_best_block(cc: ChainClient, tx: UnboundedSender<Action>) {
    let api = cc.client.clone();
    let runtime = cc.runtime.clone();
    tokio::spawn(async move {
        match api.blocks().subscribe_best().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::ChainConnection(
                                runtime.clone(),
                                ConnectionState::Connected(block.number().into()),
                            ));
                        }
                        Err(e) => {
                            // Handle disconnection errors.
                            if e.is_disconnected_will_reconnect() {
                                warn!("Lost connection to the {} RPC. Reconnecting...", cc.runtime);
                                let _ = tx.send(Action::ChainConnection(
                                    runtime.clone(),
                                    ConnectionState::Connecting,
                                ));
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
