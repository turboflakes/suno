use crate::app::Action;
use crate::config::{SupportedRuntime, CONFIG};
use crate::utils::create_substrate_rpc_client_from_url;
use crate::widgets::scrollbar::render_scrollbar;
use log::{error, info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Style, Stylize},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, Row, StatefulWidget, Table, TableState,
        Widget,
    },
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
struct ChainsListState {
    chains: Vec<ChainClient>,
    table_state: TableState,
}

#[derive(Debug, Clone)]
struct ChainClient {
    runtime: SupportedRuntime,
    client: OnlineClient<SubstrateConfig>,
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
    pub async fn run(&self, tx: UnboundedSender<Action>) {
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            let url = config.get_default_rpc_url(chain).unwrap_or_default();
            match create_substrate_rpc_client_from_url(url).await {
                Ok(rpc_client) => {
                    match OnlineClient::<SubstrateConfig>::from_rpc_client(rpc_client).await {
                        Ok(client) => {
                            let chain_client = ChainClient {
                                runtime: chain.clone(),
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

    fn on_connecting(&self, chain_client: ChainClient, tx: UnboundedSender<Action>) {
        let mut state = self.state.write().unwrap();
        state.chains.push(chain_client.clone());
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

    pub fn scroll_down(&self) {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.chains.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.scroll_down_by(1);
            }
        }
    }

    pub fn scroll_up(&self) {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                state.table_state.select_last();
            } else {
                state.table_state.scroll_up_by(1);
            }
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
}

impl Widget for &ChainsListWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let block = Block::new()
            .title(" Chains ")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let rows = state.chains.iter();
        let widths = [Constraint::Fill(1), Constraint::Length(10)];
        let table = Table::new(rows, widths)
            .block(block)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(" > ")
            .row_highlight_style(Style::new().on_blue());

        let scrollbar_area = Rect {
            y: area.y + 1,
            height: area.height - 2,
            ..area
        };

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        let row_index = state.table_state.selected().unwrap();
        render_scrollbar(row_index, state.chains.len(), scrollbar_area, buf);
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
