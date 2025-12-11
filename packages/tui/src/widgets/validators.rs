use crate::widgets::chains::ChainClient;
use crate::widgets::popup::PopupWidget;
use crate::widgets::scrollbar::render_scrollbar;
use log::{info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Rect},
    style::{Color, Modifier, Style},
    text::Text,
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, TableState, Widget},
};
use std::collections::HashMap;
use subxt::{OnlineClient, SubstrateConfig};
use suno_actions::{Action, ChainAction, SystemAction};
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
use suno_primitives::{AccountDisplay, NodeAccount, ValidatorKey};
// use suno_westend;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use subxt::utils::{AccountId32, H256};
use suno_asset_hub_paseo;
use tokio::sync::mpsc::UnboundedSender;

type Commission = u32;
type Points = u32;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ValidatorState {
    /// Validator is an authority in the active set
    Authority,
    /// Validator is an authority and also a parachain authority
    ParaAuthority,
    /// Validator is in the waiting queue
    Waiting,
    /// Validator status is unknown or not yet determined
    #[default]
    Unknown,
}
#[derive(Debug, Clone, Default)]
pub struct Stake {
    pub own: u128,
    pub nominators: u128,
    pub active: u128,
}

#[derive(Debug, Clone)]
pub struct Nominators {
    pub stash: AccountId32,
    pub stake: u128,
    pub is_backer: bool,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub account: NodeAccount,
    pub commission: Commission,
    pub stake: Stake,
    pub nominators: Vec<Nominators>,
    pub points: Points,
    pub is_next_authority: bool,
    pub is_chilled: bool,
    pub state: ValidatorState,
}

impl Validator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
            commission: 0,
            stake: Stake::default(),
            nominators: Vec::new(),
            points: 0,
            is_next_authority: false,
            is_chilled: false,
            state: ValidatorState::default(),
        }
    }

    pub fn key(&self) -> &ValidatorKey {
        &self.account.account_key()
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.account.runtime()
    }

    pub fn identity(&self) -> &Option<String> {
        self.account.identity()
    }

    pub fn commission_as_percentage(&self, decimal_places: usize) -> String {
        let percentage = self.commission as f64 / 10_000_000.0;
        let formatted = format!("{:.prec$}", percentage, prec = decimal_places);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        format!("{}%", trimmed)
    }

    pub fn chill(&self, chain_client: &ChainClient, tx: UnboundedSender<Action>) {
        if !chain_client.is_ready() {
            warn!("TODO: Chain {} not ready", chain_client.runtime());
            return;
        }

        let api = chain_client.client().clone();
        let runtime = self.runtime().clone();
        let tx = tx.clone();
        let stash = self.account.stash().clone();
        tokio::spawn(async move {
            // let response = match runtime {
            //     SupportedRuntime::Westend => {
            //         // TODO: Implement password input for proxy signing
            //         let chill_xt = suno_westend::staking::chill();
            //         suno_westend::submit_as_proxy(&api, chill_xt, stash, None, tx).await
            //     }
            //     _ => unimplemented!("Chill not implemented for {:?}", runtime),
            // };
            // match response {
            //     Err(e) => {
            //         warn!("TODO: error: {:?}", e);
            //     }
            //     _ => (),
            // }
        });
    }
}

impl AccountDisplay for Validator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }
}

#[derive(Debug, Default)]
pub struct ValidatorsListState {
    validators: HashMap<ValidatorKey, Validator>,
    validators_order: Vec<ValidatorKey>,
    table_state: TableState,
    is_active: bool,
}

impl ValidatorsListState {
    pub fn add_validator(&mut self, validator: Validator) {
        let key = validator.key();
        if !self.validators.contains_key(&key) {
            self.validators_order.push(key.clone());
        }
        self.validators.insert(key.clone(), validator);
    }

    pub fn update_validator_commission(
        &mut self,
        validator_key: &ValidatorKey,
        commission: Commission,
    ) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.commission = commission;
        }
    }

    // Helper method to get validator by table index
    pub fn get_validator_by_index(&self, index: usize) -> Option<&Validator> {
        self.validators_order
            .get(index)
            .and_then(|key| self.validators.get(key))
    }

    pub fn get_validator_by_index_cloned(&self, index: usize) -> Option<Validator> {
        self.get_validator_by_index(index).cloned()
    }

    /// Returns an iterator of validators in display order
    pub fn validators_iter(&self) -> impl Iterator<Item = &Validator> {
        self.validators_order
            .iter()
            .filter_map(move |key| self.validators.get(key))
    }
}

#[derive(Debug)]
pub struct ValidatorsListWidget {
    state: Arc<RwLock<ValidatorsListState>>,
    /// The sender to send actions to update the state to the app.
    tx: UnboundedSender<Action>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidatorsCompactWidget {
    state: Arc<RwLock<ValidatorsListState>>,
}

#[derive(Debug, Clone, Default)]
pub struct ValidatorsDetailWidget {
    state: Arc<RwLock<ValidatorsListState>>,
}

impl ValidatorsListWidget {
    // Add methods to create the alternative widgets
    pub fn as_compact(&self) -> ValidatorsCompactWidget {
        ValidatorsCompactWidget {
            state: self.state.clone(),
        }
    }

    pub fn as_detail(&self) -> ValidatorsDetailWidget {
        ValidatorsDetailWidget {
            state: self.state.clone(),
        }
    }
}

impl ValidatorsListWidget {
    pub fn new(tx: UnboundedSender<Action>) -> Self {
        Self {
            state: Arc::new(RwLock::new(ValidatorsListState::default())),
            tx,
        }
    }

    pub fn on_init(&self) {
        let mut state = self.state.write().unwrap();
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                for validator in &chain_config.validators {
                    match validator {
                        NodeConfig::Address(stash) => {
                            let validator = Validator::new(chain_name.clone(), stash.clone());
                            state.add_validator(validator.clone());

                            self.tx
                                .send(Action::Chain(ChainAction::FetchInitialValidatorData(
                                    validator.key().clone(),
                                )))
                                .unwrap_or_else(|err| self.on_err(err.into()));
                        }
                        NodeConfig::Detailed { stash, .. } => {
                            let validator = Validator::new(chain_name.clone(), stash.clone());
                            state.add_validator(validator.clone());

                            self.tx
                                .send(Action::Chain(ChainAction::FetchInitialValidatorData(
                                    validator.key().clone(),
                                )))
                                .unwrap_or_else(|err| self.on_err(err.into()));

                            // TODO: Implement command handling
                            // if let Some(cmds) = commands {
                            //     for cmd in cmds {
                            //         println!("  Command: {} ({})", cmd.name, cmd.run);
                            //     }
                            // }
                        }
                    }
                }
            }
        }
        // Select the first validator.
        if !state.validators.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    pub fn move_down(&self) -> Option<Validator> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.validators_order.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.scroll_down_by(1);
            }
            state
                .table_state
                .selected()
                .and_then(|i| state.get_validator_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Validator> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = state.validators_order.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.scroll_up_by(1);
            }
            state
                .table_state
                .selected()
                .and_then(|i| state.get_validator_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Validator> {
        let state = self.state.read().unwrap();
        state
            .table_state
            .selected()
            .and_then(|i| state.get_validator_by_index_cloned(i))
    }

    pub fn fetch_initial_validator_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        validator_key: &ValidatorKey,
        block_hash: H256,
    ) {
        let api = api.clone();
        let validator_key = validator_key.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let result = match validator_key.runtime().asset_hub_runtime() {
                SupportedRuntime::AssetHubPaseo => {
                    suno_asset_hub_paseo::fetch_initial_validator_data(
                        &api,
                        block_hash,
                        validator_key,
                        tx.clone(),
                    )
                    .await
                }
                _ => {
                    unimplemented!(
                        "fetch_initial_validator_data for runtime {:?}",
                        validator_key.runtime()
                    )
                }
            };

            if let Err(e) = result {
                let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
            }
        });
    }

    pub fn update_validator_commission(
        &self,
        validator_key: &ValidatorKey,
        commission: Commission,
    ) {
        let mut state = self.state.write().unwrap();
        state.update_validator_commission(validator_key, commission);
    }
}

// Compact widget implementation, mostly to be used on the left menu
impl Widget for &ValidatorsCompactWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let (table_style, highlight_style, highlight_symbol) = match state.is_active {
            true => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::Black).bg(Color::White),
                "❯ ",
            ),
            false => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Blue),
                "",
            ),
        };

        let block = Block::new()
            .title("Validators")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_type(BorderType::Plain);

        let rows = state.validators_iter();

        let widths = [
            Constraint::Fill(1),    // Network column
            Constraint::Length(14), // Stash column
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .style(table_style)
            .row_highlight_style(highlight_style)
            .highlight_symbol(highlight_symbol);

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        // // Render scrollbar when active
        // if state.is_active {

        //     let scrollbar_area = Rect {
        //         x: area.x,
        //         y: area.y + 1,
        //         width: 1,
        //         height: area.height - 2,
        //         ..area
        //     };
        //     if let Some(row_index) = state.table_state.selected() {
        //         render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
        //     }
        // }
    }
}

// Detailed widget implementation, with all relevant information
impl Widget for &ValidatorsDetailWidget {
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
            .borders(Borders::NONE)
            .border_type(BorderType::Plain);

        let rows = state.validators_iter().map(|v| {
            Row::new(vec![
                Text::from(v.to_compact_string(5)).alignment(Alignment::Left),
                Text::from(v.commission_as_percentage(2)),
            ])
        });

        let widths = [
            Constraint::Length(20), // Stash Column
            Constraint::Fill(1),    // TODO Column
        ];

        let table = Table::new(rows, widths)
            // .block(block)
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, &mut state.table_state);
    }
}

impl From<&Validator> for Row<'_> {
    fn from(v: &Validator) -> Self {
        let v = v.clone();
        Row::new(vec![
            Text::from(v.runtime().to_string()),
            Text::from(v.to_compact_string(5)).alignment(Alignment::Right),
        ])
    }
}
