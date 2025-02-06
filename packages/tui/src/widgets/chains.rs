use crate::config::{SupportedRuntime, CONFIG};
use crate::utils::create_substrate_rpc_client_from_url;
use crate::widgets::scrollbar::render_scrollbar;
use log::error;
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
enum ConnectionState {
    #[default]
    Idle,
    Connecting,
    Connected,
    Error(String),
}

impl ChainsListWidget {
    /// Initialize OnlineClients for each configured chain.
    pub async fn run(&self) {
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
                                state: ConnectionState::Connected,
                            };
                            self.on_connected(chain_client)
                        }
                        Err(err) => self.on_err(Box::new(err)),
                    }
                }
                Err(err) => self.on_err(err),
            }
        }
    }

    fn on_connected(&self, chain_client: ChainClient) {
        let mut state = self.state.write().unwrap();
        state.chains.push(chain_client);
        if !state.chains.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        error!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    pub fn scroll_down(&self) {
        self.state.write().unwrap().table_state.scroll_down_by(1);
    }

    pub fn scroll_up(&self) {
        self.state.write().unwrap().table_state.scroll_up_by(1);
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
        let widths = [Constraint::Fill(1), Constraint::Length(5)];
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
        Row::new(vec![cc.runtime.to_string(), "✓".to_string()])
    }
}
