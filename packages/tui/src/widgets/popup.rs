use crate::menu::{AsChar, Command, Entry, ToDescription};
use crate::theme::THEME;
use log::{info, warn};
use ratatui::style::Styled;
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Row, StatefulWidget, Table, TableState, Widget},
};
use snops_config::CONFIG;
use std::sync::{Arc, RwLock};
use std::time::Instant;

/// Popup modes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Menu,
    Confirm,
    Transaction,
}

// Popup Call definitions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Staking {
    Chill,
    Bond,
    Unbond,
    ChangeRewardDestination,
    ChangeCommission,
    KickNominators,
    SetSessionKey,
}

impl AsChar for Staking {
    fn as_char(&self) -> char {
        match self {
            Self::Chill => 'c',
            Self::Bond => 'b',
            Self::Unbond => 'u',
            Self::ChangeRewardDestination => 'r',
            Self::ChangeCommission => 'f',
            Self::KickNominators => 'k',
            Self::SetSessionKey => 's',
        }
    }
}

impl ToDescription for Staking {
    fn description(&self) -> String {
        match self {
            Self::Chill => "Declare no intention to validate".to_string(),
            Self::Bond => "Bond more funds".to_string(),
            Self::Unbond => "Unbond funds".to_string(),
            Self::ChangeRewardDestination => "Change reward destination".to_string(),
            Self::ChangeCommission => "Change commission".to_string(),
            Self::KickNominators => "Kick nominators".to_string(),
            Self::SetSessionKey => "Change session keys".to_string(),
        }
    }
}

impl std::fmt::Display for Staking {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Chill => write!(f, "staking.chill"),
            Self::Bond => write!(f, "staking.bond"),
            Self::Unbond => write!(f, "staking.unbond"),
            Self::ChangeRewardDestination => write!(f, "staking.change_reward_destination"),
            Self::ChangeCommission => write!(f, "staking.change_commission"),
            Self::KickNominators => write!(f, "staking.kick_nominators"),
            Self::SetSessionKey => write!(f, "staking.set_session_key"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PopupWidget {
    state: Arc<RwLock<ListState>>,
}

#[derive(Debug)]
struct ListState {
    options: Vec<Entry<Staking>>,
    table_state: TableState,
    is_visible: bool,
    mode: Mode,
    spinner_frames: Vec<&'static str>,
    spinner_start_time: Instant,
    spinner_counter: usize,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            table_state: TableState::default(),
            is_visible: false,
            mode: Mode::default(),
            spinner_frames: vec!["⠋", "⠙", "⠹", "⠸", "⢸", "⣸", "⣠", "⣄", "⣇", "⠇", "⠏"],
            spinner_start_time: Instant::now(),
            spinner_counter: 0,
        }
    }
}

impl ListState {
    fn spinner_frame(&self) -> &str {
        let elapsed = self.spinner_start_time.elapsed().as_millis() as u64;
        let frame_index = (elapsed / 250) as usize % self.spinner_frames.len();
        self.spinner_frames[frame_index]
    }

    fn spinner_progress(&self) -> String {
        let full = "⣿".repeat(self.spinner_counter);
        format!("⣿{}{}", full, self.spinner_frame())
    }
}

impl PopupWidget {
    pub fn on_init(&self, mode: Mode, call: Option<Staking>) {
        let mut state = self.state.write().unwrap();
        state.options.clear();
        state.mode = mode.clone();
        match mode {
            Mode::Menu => self.init_menu(&mut state),
            Mode::Confirm => {
                if call.is_none() {
                    self.on_err("No call provided for confirmation mode".into());
                    return;
                }
                self.init_confirmation(&mut state, call.unwrap())
            }
            Mode::Transaction => self.init_transaction(&mut state),
        }

        // Select the first option.
        if !state.options.is_empty() {
            state.table_state.select(Some(0));
        }

        //
        state.is_visible = true;
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    fn init_transaction(&self, state: &mut ListState) {
        state.spinner_start_time = Instant::now();
        state.spinner_counter = 0;
        state
            .options
            .push(Entry::new(Command::Text("broadcasted".to_string())));
    }

    fn init_menu(&self, state: &mut ListState) {
        // Note: match entries with the keys defined in the `handle_key_events` function.
        state
            .options
            .push(Entry::new(Command::Instruction(Staking::Chill)));
        state
            .options
            .push(Entry::new(Command::Instruction(Staking::Bond)));
        state.options.push(Entry::new(Command::Instruction(
            Staking::ChangeRewardDestination,
        )));
        state
            .options
            .push(Entry::new(Command::Instruction(Staking::ChangeCommission)));
        state
            .options
            .push(Entry::new(Command::Instruction(Staking::KickNominators)));
        state
            .options
            .push(Entry::new(Command::Instruction(Staking::SetSessionKey)));
    }

    fn init_confirmation(&self, state: &mut ListState, call: Staking) {
        match call {
            Staking::Chill => self.init_chill(state),
            // Staking::Bond => self.init_bond(state),
            // Staking::Unbond => self.init_unbond(state),
            // Staking::ChangeRewardDestination => self.init_change_reward_destination(state),
            // Staking::ChangeCommission => self.init_change_commission(state),
            // Staking::KickNominators => self.init_kick_nominators(state),
            // Staking::SetSessionKey => self.init_set_session_key(state),
            _ => {
                warn!("Unsupported call: {:?}", call);
                return;
            }
        }
    }

    fn init_chill(&self, state: &mut ListState) {
        state
            .options
            .push(Entry::new(Command::Instruction(Staking::Chill)));
        state
            .options
            .push(Entry::new(Command::Text("cancel".to_string())));
    }

    pub fn move_down(&self) -> Option<Entry<Staking>> {
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

    pub fn move_up(&self) -> Option<Entry<Staking>> {
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

    pub fn is_visible(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_visible
    }

    pub fn hide(&self) {
        let mut state = self.state.write().unwrap();
        state.is_visible = false;
    }

    pub fn get_selected(&self) -> Option<Entry<Staking>> {
        let state = self.state.read().unwrap();
        state
            .table_state
            .selected()
            .map(|i| state.options[i].clone())
    }

    pub fn get_mode(&self) -> Mode {
        let state = self.state.read().unwrap();
        state.mode.clone()
    }

    pub fn show_menu(&self) {
        self.on_init(Mode::Menu, None);
    }

    pub fn show_transaction(&self) {
        self.on_init(Mode::Transaction, None);
    }

    pub fn update_transaction_status(&self, message: String) {
        let mut state = self.state.write().unwrap();
        state.spinner_counter += 1;
        state.options.clear();
        state.options.push(Entry::new(Command::Text(message)));
    }

    pub fn confirm_chill_attempt(&self) {
        self.on_init(Mode::Confirm, Some(Staking::Chill));
    }
}

impl Widget for &PopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        if !state.is_visible {
            return; // Do not render if popup is not active.
        }

        match state.mode {
            Mode::Menu => render_menu(area, buf, &mut state),
            Mode::Confirm => render_confirmation(area, buf, &mut state),
            Mode::Transaction => render_transaction(area, buf, &mut state),
        }
    }
}

fn render_menu(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    let block = Block::new()
        .title(" Commands ")
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let rows = state
        .options
        .iter()
        .map(|f| f.to_row(state.mode.clone(), None));
    let widths = [
        Constraint::Length(4),
        Constraint::Fill(1),
        Constraint::Fill(2),
    ];

    let table = Table::new(rows, widths)
        .style(THEME.table.base(state.is_visible))
        .block(block)
        .header(Row::new(vec!["Key", "Extrinsic", "Description"]).set_style(THEME.table.header))
        .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, area, buf, &mut state.table_state);
}

fn render_confirmation(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    let block = Block::new()
        .title(" Confirm ")
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let rows = state
        .options
        .iter()
        .map(|f| f.to_row(state.mode.clone(), None));
    let widths = [Constraint::Length(24), Constraint::Fill(1)];
    let table = Table::new(rows, widths)
        .style(THEME.table.base(state.is_visible))
        .block(block)
        .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, area, buf, &mut state.table_state);
}

fn render_transaction(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Plain);

    let spinner_progress = state.spinner_progress();
    let rows = state
        .options
        .iter()
        .map(|f| f.to_row(state.mode.clone(), Some(&spinner_progress)));
    let widths = [Constraint::Length(8), Constraint::Fill(1)];
    let table = Table::new(rows, widths)
        .style(THEME.table.base(state.is_visible))
        .block(block);
    // .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, area, buf, &mut state.table_state);
}

impl<T: AsChar + std::fmt::Display + ToDescription + Clone> Entry<T> {
    pub fn to_row(&self, mode: Mode, msg: Option<&str>) -> Row<'_> {
        let command = self.get_command();
        match command {
            Command::Instruction(c) => {
                let mut row_data = Vec::new();

                // Add menu-specific formatting
                match mode {
                    Mode::Menu => {
                        row_data.push(c.as_char().to_string());
                        row_data.push(c.to_string());
                        row_data.push(c.description());
                    }
                    Mode::Confirm => {
                        row_data.push(c.to_string());
                        row_data.push(c.description());
                    }
                    _ => {}
                }

                Row::new(row_data)
            }
            Command::Text(t) => match mode {
                Mode::Transaction => Row::new(vec![
                    msg.unwrap_or("").to_string(),
                    format!("Transaction {t}"),
                ]),
                _ => Row::new(vec![t.to_string()]),
            },
        }
    }
}
