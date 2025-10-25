use crate::utils::create_substrate_rpc_client_from_url;
use crate::widgets::scrollbar::render_scrollbar;
use log::{error, info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{
        Block, BorderType, Borders, Cell, Clear, Row, StatefulWidget, Table, TableState, Widget,
    },
};
use snops_actions::{network::ConnectionState, Action, ChainAction, SystemAction};
use snops_config::{SupportedRuntime, CONFIG};
use std::sync::{Arc, RwLock};
use subxt::{OnlineClient, SubstrateConfig};
use tokio::sync::mpsc::UnboundedSender;

pub type BlockNumber = u32;

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
    pub runtime: SupportedRuntime,
    pub client: OnlineClient<SubstrateConfig>,
    state: ConnectionState,
}

impl ChainClient {
    pub fn is_ready(&self) -> bool {
        matches!(self.state, ConnectionState::Connected(_))
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
                                let chain_client = ChainClient {
                                    runtime: chain_name.clone(),
                                    client,
                                    state: ConnectionState::Connecting,
                                };
                                self.on_connecting(chain_client);
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

    fn on_connecting(&self, chain_client: ChainClient) {
        let mut state = self.state.write().unwrap();
        state.chains.push(chain_client.clone());
        // Select the first chain.
        if !state.chains.is_empty() {
            state.table_state.select(Some(0));
        }
        // Launch a task to subscribe the head of the chain.
        subscribe_best_block(chain_client, self.tx.clone());
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        self.tx
            .send(Action::System(SystemAction::Error(
                "some error".to_string(),
            )))
            .unwrap();

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

        // if state.is_active {
        //     // Render scrollbar.
        //     let scrollbar_area = Rect {
        //         y: area.y + 1,
        //         height: area.height - 2,
        //         ..area
        //     };
        //     if let Some(row_index) = state.table_state.selected() {
        //         render_scrollbar(row_index, state.chains.len(), scrollbar_area, buf);
        //     }
        // }
    }
}

impl From<&ChainClient> for Row<'_> {
    fn from(cc: &ChainClient) -> Self {
        let cc = cc.clone();
        Row::new(vec![
            Text::from(cc.runtime.to_string()),
            Text::from(cc.state.to_string()).alignment(Alignment::Right),
        ])
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
                            let _ = tx.send(Action::Chain(ChainAction::Connection {
                                runtime: runtime.clone(),
                                state: ConnectionState::Connected(block.number().into()),
                            }));
                        }
                        Err(e) => {
                            // Handle disconnection errors.
                            if e.is_disconnected_will_reconnect() {
                                info!("Lost connection to {} reconnecting...", cc.runtime);
                                let _ = tx.send(Action::Chain(ChainAction::Connection {
                                    runtime: runtime.clone(),
                                    state: ConnectionState::Connecting,
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
