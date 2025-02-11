use crate::config::CONFIG;
use crate::menu::Entry;
use crate::{app::Action, menu::Command};
use log::{info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, TableState, Widget},
};
use std::sync::{Arc, RwLock};

/// Popup variations.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Variation {
    #[default]
    Menu,
    Chill,
    Bond,
    Unbond,
    ChangeRewardDestination,
    ChangeCommission,
    KickNominators,
    SetSessionKey,
}

#[derive(Debug, Clone, Default)]
pub struct ValidatorsPopupWidget {
    state: Arc<RwLock<ListState>>,
}

#[derive(Debug, Default)]
struct ListState {
    options: Vec<Entry>,
    table_state: TableState,
    is_active: bool,
    variation: Variation,
}

impl ValidatorsPopupWidget {
    pub fn on_init(&self, variation: Variation, active: bool) {
        let mut state = self.state.write().unwrap();
        state.options.clear();
        state.is_active = active;
        state.variation = variation;
        match state.variation {
            Variation::Menu => self.init_menu(&mut state),
            Variation::Chill => self.init_chill(&mut state),
            // Variation::Bond => self.init_bond(&mut state),
            // Variation::Unbond => self.init_unbond(&mut state),
            // Variation::ChangeRewardDestination => self.init_change_reward_destination(&mut state),
            // Variation::ChangeCommission => self.init_change_commission(&mut state),
            // Variation::KickNominators => self.init_kick_nominators(&mut state),
            // Variation::SetSessionKey => self.init_set_session_key(&mut state),
            _ => {
                warn!("Unsupported variation: {:?}", state.variation);
                return;
            }
        }

        // Select the first option.
        if !state.options.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    fn init_menu(&self, state: &mut ListState) {
        // Note: match entries with the keys defined in the `handle_key_events` function.
        state.options.push(Entry::new(
            Command::Char('c'),
            "chill validator".to_string(),
        ));
        state.options.push(Entry::new(
            Command::Char('b'),
            "bond more funds".to_string(),
        ));
        state.options.push(Entry::new(
            Command::Char('r'),
            "change reward destination".to_string(),
        ));
        state.options.push(Entry::new(
            Command::Char('f'),
            "change commission".to_string(),
        ));
        state.options.push(Entry::new(
            Command::Char('k'),
            "kick nominators".to_string(),
        ));
        state.options.push(Entry::new(
            Command::Char('s'),
            "change session keys".to_string(),
        ));
    }

    fn init_chill(&self, state: &mut ListState) {
        state.options.push(Entry::new(
            Command::Instruction("staking.chill".to_string()),
            "chill validator".to_string(),
        ));
        state.options.push(Entry::new(
            Command::Instruction("cancel".to_string()),
            "bond more funds".to_string(),
        ));
    }

    pub fn move_down(&self) -> Option<Entry> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.options.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.scroll_down_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.options[i].clone())
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Entry> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = state.options.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.scroll_up_by(1);
            }
            state
                .table_state
                .selected()
                .map(|i| state.options[i].clone())
        } else {
            None
        }
    }

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Entry> {
        let state = self.state.read().unwrap();
        state
            .table_state
            .selected()
            .map(|i| state.options[i].clone())
    }

    pub fn menu(&self) {
        self.on_init(Variation::Menu, false);
    }

    pub fn chill_attempt(&self) {
        info!("Chill attempt");
        // let mut state = self.state.write().unwrap();
        // state.variation = Variation::Chill;
        self.on_init(Variation::Chill, true);
        // state.is_active = true;
    }
}

impl Widget for &ValidatorsPopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        if !state.is_active {
            return; // Do not render if popup is not active.
        }

        let (table_style, highlight_style) = match state.is_active {
            true => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::Black).bg(Color::White),
            ),
            false => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::White),
            ),
        };

        let block = Block::new()
            .title(" Menu ")
            .borders(Borders::ALL)
            .border_type(BorderType::Plain);

        let rows = state.options.iter();
        let widths = [Constraint::Length(6), Constraint::Fill(1)];
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
        //     let row_index = state.table_state.selected().unwrap();
        //     render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
        // }
    }
}

impl From<&Entry> for Row<'_> {
    fn from(o: &Entry) -> Self {
        let o = o.clone();
        Row::new(vec![o.command().to_string(), o.description().to_string()])
    }
}
