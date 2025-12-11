use crate::node_account::{AccountDisplay, NodeAccount};
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
use subxt::{OnlineClient, SubstrateConfig};
use suno_actions::{Action, ChainAction, SystemAction};
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
// use suno_westend;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use subxt::utils::{AccountId32, H256};
use suno_asset_hub_paseo;
use tokio::sync::mpsc::UnboundedSender;

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

#[derive(Debug, Default)]
pub struct ValidatorsListState {
    validators: Vec<Validator>,
    table_state: TableState,
    is_active: bool,
}

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
    pub commission: u128,
    pub stake: Stake,
    pub nominators: Vec<Nominators>,
    pub points: u32,
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

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.account.runtime()
    }

    pub fn identity(&self) -> &Option<String> {
        self.account.identity()
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
    fn stash(&self) -> &AccountId32 {
        &self.account.stash()
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
                            state
                                .validators
                                .push(Validator::new(chain_name.clone(), stash.clone()));

                            // Send a message to fetch initial validator chain data.
                            self.tx
                                .send(Action::Chain(ChainAction::FetchInitialValidatorData(
                                    chain_name.clone(),
                                    stash.clone(),
                                )))
                                .unwrap_or_else(|err| self.on_err(err.into()));
                        }
                        NodeConfig::Detailed { stash, .. } => {
                            state
                                .validators
                                .push(Validator::new(chain_name.clone(), stash.clone()));

                            // Send a message to fetch initial validator chain data.
                            self.tx
                                .send(Action::Chain(ChainAction::FetchInitialValidatorData(
                                    chain_name.clone(),
                                    stash.clone(),
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
            if selected == state.validators.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.scroll_down_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.validators[i].clone())
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Validator> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = state.validators.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.scroll_up_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.validators[i].clone())
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
            .map(|i| state.validators[i].clone())
    }

    pub fn fetch_initial_validator_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        runtime: SupportedRuntime,
        block_hash: H256,
        stash: AccountId32,
    ) {
        let api = api.clone();
        let runtime = runtime.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let result = match runtime.asset_hub_runtime() {
                SupportedRuntime::AssetHubPaseo => {
                    suno_asset_hub_paseo::fetch_initial_validator_data(
                        api,
                        block_hash,
                        stash,
                        tx.clone(),
                    )
                    .await
                }
                _ => {
                    unimplemented!("fetch_initial_validator_data for runtime {:?}", runtime)
                }
            };

            if let Err(e) = result {
                let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
            }
        });
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

        let rows = state.validators.iter();

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

        let validators = state.validators.clone();
        let rows = validators.iter().map(|v| {
            Row::new(vec![
                Text::from(v.to_compact_string(5)).alignment(Alignment::Left),
                Text::from(">TODO<"),
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
