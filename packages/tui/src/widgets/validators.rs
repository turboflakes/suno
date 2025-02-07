use crate::app::Action;
use crate::config::CONFIG;
use crate::widgets::chains::ChainClient;
use crate::widgets::scrollbar::render_scrollbar;
use log::warn;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{
        Block, BorderType, Borders, HighlightSpacing, Row, StatefulWidget, Table, TableState,
        Widget,
    },
};
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
    stash: AccountId32,
    identity: Option<String>,
}

impl Validator {
    pub fn new(stash: AccountId32) -> Self {
        Self {
            stash,
            identity: None,
        }
    }
}

impl ValidatorsListWidget {
    pub fn on_chain_selected(&self, chain_client: ChainClient, _tx: UnboundedSender<Action>) {
        let mut state = self.state.write().unwrap();
        let config = CONFIG.clone();
        for key in config.validators.iter() {
            if let Some(validators) = key.chain.get(&chain_client.runtime) {
                for value in validators.iter() {
                    let stash = match AccountId32::from_str(&value) {
                        Ok(stash) => stash,
                        Err(err) => {
                            self.on_err(Box::new(err));
                            continue;
                        }
                    };
                    state.validators.push(Validator::new(stash));
                }
                break;
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

    pub fn scroll_down(&self) {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.validators.len() - 1 {
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

        let styles = if state.is_active {
            let highlight_style = Style::new().fg(Color::Black).bg(Color::White);
            let table_style = Style::new().fg(Color::White).bg(Color::Black);
            (table_style, highlight_style)
        } else {
            let table_style = Style::new().fg(Color::Blue).bg(Color::Black);
            let highlight_style = Style::new().fg(Color::White).bg(Color::Black);
            (table_style, highlight_style)
        };

        let block = Block::new()
            .title(" Validators ")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let rows = state.validators.iter();
        let widths = [Constraint::Fill(1), Constraint::Length(10)];
        let table = Table::new(rows, widths)
            .block(block)
            .style(styles.0)
            .highlight_spacing(HighlightSpacing::Always)
            .highlight_symbol(" > ")
            .row_highlight_style(styles.1);

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        if state.is_active {
            // Render scrollbar.
            let scrollbar_area = Rect {
                y: area.y + 1,
                height: area.height - 2,
                ..area
            };
            let row_index = state.table_state.selected().unwrap();
            render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
        }
    }
}

impl From<&Validator> for Row<'_> {
    fn from(cc: &Validator) -> Self {
        let cc = cc.clone();
        Row::new(vec![cc.stash.to_string()])
    }
}
