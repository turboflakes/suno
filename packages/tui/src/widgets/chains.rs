use crate::bridge::subscribe::{subscribe_best_block, subscribe_finalized_block};
use crate::utils::create_rpc_client_from_config;
use crate::widgets::scrollbar::render_scrollbar;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::Styled,
    text::Text,
    widgets::{Block, Cell, Padding, Row, StatefulWidget, Table, TableState, Widget},
};
use sp_arithmetic::Permill;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::{lightclient::LightClient, utils::H256, OnlineClient};
use suno_actions::{Action, SystemAction};
use suno_config::{CustomConfig, SupportedRuntime, CONFIG};
use suno_primitives::{
    display::{create_progress_bar_by_millis, format_millis, get_elapsed_millis},
    network::ConnectionState,
    BlockHash, BlockNumber, Chain, Epoch, Era,
};
use tokio::sync::mpsc::UnboundedSender;
use tracing::debug;

type ChainKey = SupportedRuntime;

#[derive(Debug, Default)]
pub struct ChainsList {
    chains: HashMap<ChainKey, Chain>,
    chains_order: Vec<ChainKey>,
    table_state: TableState,
    is_active: bool,
}

impl ChainsList {
    pub fn add_chain(&mut self, chain: Chain) {
        let key = chain.key();
        if !self.chains.contains_key(&key) {
            self.chains_order.push(key);
        }
        self.chains.insert(key, chain);
    }

    pub fn set_best_block(&mut self, chain_key: &ChainKey, block_number: BlockNumber) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            if chain.best_block() != block_number {
                chain.set_best_block(block_number);
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
            if chain.finalized_block() != block_number {
                chain.set_finalized_block(block_number);
                chain.set_finalized_block_hash(Some(block_hash));
                chain.set_finalized_block_ts(
                    SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_millis(),
                );
                return true;
            }
        }
        false
    }

    pub fn set_connection_state(&mut self, chain_key: &ChainKey, state: ConnectionState) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            if chain.state() != &state {
                chain.set_state(state);
                return true;
            }
        }
        false
    }

    pub fn set_era(&mut self, chain_key: &ChainKey, data: Era) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.set_era(Some(data));
            return true;
        }
        false
    }

    pub fn set_epoch(&mut self, chain_key: &ChainKey, data: Epoch) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.set_epoch(Some(data));
            return true;
        }
        false
    }

    pub fn set_active_vals(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.set_active_vals(counter);
            return true;
        }
        false
    }

    pub fn set_total_vals(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.set_total_vals(counter);
            return true;
        }
        false
    }

    pub fn set_active_noms(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.set_active_noms(counter);
            return true;
        }
        false
    }

    pub fn set_total_noms(&mut self, chain_key: &ChainKey, counter: u32) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.set_total_noms(counter);
            return true;
        }
        false
    }

    pub fn set_total_staked(&mut self, chain_key: &ChainKey, value: Permill) -> bool {
        if let Some(chain) = self.chains.get_mut(chain_key) {
            chain.set_total_staked_pm(value);
            return true;
        }
        false
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn _get_chain_by_key(&self, chain_key: ChainKey) -> Option<&Chain> {
        self.chains.get(&chain_key)
    }

    pub fn get_chain_by_key_cloned(&self, chain_key: ChainKey) -> Option<Chain> {
        self.chains.get(&chain_key).cloned()
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

    pub fn _get_selected_ref(&self) -> Option<&Chain> {
        self.table_state
            .selected()
            .and_then(|i| self.get_chain_by_index(i))
    }

    /// Initialize OnlineClients for each configured chain.
    pub async fn on_init(&mut self, tx: UnboundedSender<Action>) {
        let config = CONFIG.clone();
        let mut relay_lc: Option<(SupportedRuntime, LightClient)> = None;
        for chain in config.chains.iter() {
            for (runtime, chain_config) in chain {
                let (rpc_client, new_relay_lc) =
                    match create_rpc_client_from_config(runtime, chain_config, relay_lc.clone())
                        .await
                    {
                        Ok(result) => result,
                        Err(err) => {
                            Self::error(&tx, err);
                            continue;
                        }
                    };

                relay_lc = new_relay_lc.or(relay_lc);

                let client = match OnlineClient::<CustomConfig>::from_rpc_client(rpc_client).await {
                    Ok(client) => client,
                    Err(err) => {
                        Self::error(&tx, err.into());
                        continue;
                    }
                };

                let mut chain = Chain::new(*runtime, client);
                if let Err(err) = chain.validate_genesis() {
                    Self::error(&tx, err.into());
                }
                self.add_chain(chain.clone());
                if chain.is_validated() {
                    subscribe_best_block(&chain, tx.clone());
                    subscribe_finalized_block(&chain, tx.clone());
                }
            }
        }
        self.init_table();
    }

    fn error(tx: &UnboundedSender<Action>, err: Box<dyn std::error::Error>) {
        tx.send(Action::System(SystemAction::Error(err.to_string())))
            .expect("Failed to send error message");
    }

    pub fn subscribe_best_block(&self, chain: &Chain, tx: UnboundedSender<Action>) {
        subscribe_best_block(chain, tx);
    }

    pub fn subscribe_finalized_block(&self, chain: &Chain, tx: UnboundedSender<Action>) {
        subscribe_finalized_block(chain, tx);
    }

    pub fn move_down(&mut self) -> Option<Chain> {
        if let Some(selected) = self.table_state.selected() {
            if selected == self.chains.len() - 1 {
                self.table_state.select_first();
            } else {
                self.table_state.scroll_down_by(1);
            }
            self.table_state
                .selected()
                .and_then(|i| self.get_chain_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn move_up(&mut self) -> Option<Chain> {
        if let Some(selected) = self.table_state.selected() {
            if selected == 0 {
                let i = self.chains.len() - 1;
                self.table_state.select(Some(i));
            } else {
                self.table_state.scroll_up_by(1);
            }
            self.table_state
                .selected()
                .and_then(|i| self.get_chain_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn init_table(&mut self) {
        if !self.chains.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Chain> {
        self.table_state
            .selected()
            .and_then(|i| self.get_chain_by_index_cloned(i))
    }

    pub fn get_chain_by_runtime(&self, runtime: SupportedRuntime) -> Option<Chain> {
        self.get_chain_by_key_cloned(runtime)
    }

    pub fn get_api_and_block_hash(
        &self,
        runtime: SupportedRuntime,
    ) -> Option<(OnlineClient<CustomConfig>, H256)> {
        let chain = self.get_chain_by_runtime(runtime)?;

        if !chain.is_connected() {
            debug!("Chain {} not connected", runtime);
            return None;
        }

        let block_hash = chain.block_hash()?;
        Some((chain.client().clone(), block_hash))
    }

    pub fn update_connection_state(
        &mut self,
        chain_key: &ChainKey,
        connection_state: ConnectionState,
    ) -> bool {
        self.set_connection_state(chain_key, connection_state)
    }

    pub fn update_best_block(&mut self, chain_key: &ChainKey, block_number: BlockNumber) -> bool {
        self.set_best_block(chain_key, block_number)
    }

    pub fn update_finalized_block(
        &mut self,
        chain_key: &ChainKey,
        block_number: BlockNumber,
        block_hash: BlockHash,
    ) -> bool {
        self.set_finalized_block(chain_key, block_number, block_hash)
    }

    pub fn update_era(&mut self, chain_key: &ChainKey, era: Era) -> bool {
        self.set_era(chain_key, era)
    }

    pub fn update_epoch(&mut self, chain_key: &ChainKey, epoch: Epoch) -> bool {
        self.set_epoch(chain_key, epoch)
    }

    pub fn update_active_validators(&mut self, chain_key: &ChainKey, count: u32) -> bool {
        self.set_active_vals(chain_key, count)
    }

    pub fn update_total_validators(&mut self, chain_key: &ChainKey, count: u32) -> bool {
        self.set_total_vals(chain_key, count)
    }

    pub fn update_active_nominators(&mut self, chain_key: &ChainKey, count: u32) -> bool {
        self.set_active_noms(chain_key, count)
    }

    pub fn update_total_nominators(&mut self, chain_key: &ChainKey, count: u32) -> bool {
        self.set_total_noms(chain_key, count)
    }

    pub fn update_total_staked(&mut self, chain_key: &ChainKey, value: Permill) -> bool {
        self.set_total_staked(chain_key, value)
    }
}

impl Widget for &mut ChainsList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();

        let block = Block::new()
            .set_style(theme.block.pane_header(self.is_active))
            .padding(Padding::symmetric(0, 1));

        let rows = self.chains_iter().map(|chain| {
            let elapsed = get_elapsed_millis(chain.finalized_block_ts());
            let progress = create_progress_bar_by_millis(elapsed, 6);

            Row::new(vec![
                Text::from(""),
                Text::from(format!("{}{}", chain.state(), chain.runtime())),
                Text::from(format!("#{}", chain.best_block())).alignment(Alignment::Right),
                Text::from(format!("#{}", chain.finalized_block())).alignment(Alignment::Right),
                Text::from(progress.to_string()).alignment(Alignment::Right),
                Text::from(format_millis(elapsed, false)).alignment(Alignment::Right),
                Text::from(""),
            ])
        });

        let widths = [
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Length(6),
            Constraint::Length(3),
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
            .header(Row::new(header_cells).set_style(theme.table.header(self.is_active)))
            .style(theme.table.base)
            .row_highlight_style(theme.table.row_highlight(self.is_active))
            .highlight_symbol(theme.table.highlight_symbol(self.is_active));

        StatefulWidget::render(table, area, buf, &mut self.table_state);

        // Render scrollbar when active
        if self.is_active && self.chains.len() >= area.height.saturating_sub(2) as usize {
            let scrollbar_area = Rect {
                x: area.x + area.width.saturating_sub(1),
                y: area.y + 1,
                width: 1,
                height: area.height.saturating_sub(2),
            };
            if let Some(row_index) = self.table_state.selected() {
                render_scrollbar(row_index, self.chains.len(), scrollbar_area, buf);
            }
        }
    }
}
