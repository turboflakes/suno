use crate::node_account::{AccountDisplay, NodeAccount};
use crate::widgets::chains::ChainClient;
use crate::widgets::popup::PopupWidget;
use crate::widgets::scrollbar::render_scrollbar;
use log::{info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, TableState, Widget},
};
use snops_actions::Action;
use snops_config::{NodeConfig, SupportedRuntime, CONFIG};

// use snops_westend;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use subxt::utils::AccountId32;
use tokio::sync::mpsc::UnboundedSender;

#[derive(Debug, Clone, Default)]
pub struct ValidatorsListWidget {
    state: Arc<RwLock<ValidatorsListState>>,
}

#[derive(Debug, Default)]
pub struct ValidatorsListState {
    validators: Vec<Validator>,
    table_state: TableState,
    is_active: bool,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub account: NodeAccount,
}

impl Validator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
        }
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.account.runtime
    }

    pub fn identity(&self) -> Option<&String> {
        self.account.identity.as_ref()
    }

    pub fn chill(&self, chain_client: &ChainClient, tx: UnboundedSender<Action>) {
        if !chain_client.is_ready() {
            warn!("TODO: Chain {} not ready", chain_client.runtime);
            return;
        }

        let api = chain_client.client.clone();
        let runtime = self.runtime().clone();
        let tx = tx.clone();
        let stash = self.account.stash.clone();
        tokio::spawn(async move {
            // let response = match runtime {
            //     SupportedRuntime::Westend => {
            //         // TODO: Implement password input for proxy signing
            //         let chill_xt = snops_westend::staking::chill();
            //         snops_westend::submit_as_proxy(&api, chill_xt, stash, None, tx).await
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
        &self.account.stash
    }
}

impl ValidatorsListWidget {
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
                        }
                        NodeConfig::Detailed { stash, commands } => {
                            state
                                .validators
                                .push(Validator::new(chain_name.clone(), stash.clone()));
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
}

impl Widget for &ValidatorsListWidget {
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
            .title(" Validators ")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let rows = state.validators.iter();
        let widths = [Constraint::Fill(1), Constraint::Length(14)];
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
            if let Some(row_index) = state.table_state.selected() {
                render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
            }
        }
    }
}

impl From<&Validator> for Row<'_> {
    fn from(v: &Validator) -> Self {
        let v = v.clone();
        Row::new(vec![v.runtime().to_string(), v.to_compact_string(5)])
    }
}
