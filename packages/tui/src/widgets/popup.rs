use crate::call::Call;
use crate::entry::{AsChar, Command, Entry, ToDescription, ToHex, ToPlaceholder};
use crate::theme::THEME;
use crate::widgets::input_field::InputFieldWidget;
use log::{info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Position, Rect},
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Padding, Paragraph, Row, StatefulWidget, Table,
        TableState, Widget, Wrap,
    },
};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use suno_primitives::tx::Bytes;

/// Popup modes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Menu,
    Details,
    Confirm,
    Transaction,
}

#[derive(Debug, Clone, Default)]
pub struct PopupWidget {
    pub state: Arc<RwLock<ListState>>,
}

#[derive(Debug)]
pub struct ListState {
    options: Vec<Entry<Call>>,
    table_state: TableState,
    is_visible: bool,
    mode: Mode,
    spinner_frames: Vec<&'static str>,
    spinner_start_time: Instant,
    spinner_counter: usize,
    input: InputFieldWidget,
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
            input: InputFieldWidget::new(),
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
        format!("{}{}⣿", self.spinner_frame(), full)
    }

    pub fn get_input_cursor_position(&self) -> Option<Position> {
        self.input.get_cursor_position()
    }

    pub fn get_selected(&self) -> Option<Entry<Call>> {
        self.table_state.selected().map(|i| self.options[i].clone())
    }

    pub fn get_options_filtered(&self) -> Vec<Entry<Call>> {
        let input_value = self.input.value();
        let input_command = match input_value.split_once(' ') {
            None => input_value.as_str(),
            Some((command, _)) => command,
        };

        self.options
            .iter()
            .filter(|e| e.command().to_lowercase().starts_with(&input_command))
            .cloned()
            .collect()
    }

    pub fn get_selected_call(&self) -> Option<Call> {
        let options = self.get_options_filtered();
        options
            .iter()
            .next()
            .map(|e| match e.get_command() {
                Command::Instruction(call) => Some(call),
                _ => None,
            })
            .flatten()
    }
}

impl PopupWidget {
    pub fn on_init(&self, mode: Mode, call: Option<Call>) {
        let mut state = self.state.write().unwrap();
        state.options.clear();
        state.mode = mode.clone();
        match mode {
            Mode::Menu => self.init_menu(&mut state),
            Mode::Details => {
                if call.is_none() {
                    self.on_err("No call provided for details mode".into());
                    return;
                }
                self.update_details(&mut state, call.unwrap())
            }
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
        state.options.push(Entry::new(Command::Text(
            "processing transaction".to_string(),
        )));
    }

    fn init_menu(&self, state: &mut ListState) {
        state
            .options
            .push(Entry::new(Command::Instruction(Call::Bond)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::ChangeCommission)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::ChangePayee)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::Chill(Bytes::new()))));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::KickNominators)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::SetSessionKey)));
    }

    fn update_details(&self, state: &mut ListState, call: Call) {
        match call {
            Call::Chill(bytes) => {
                // self.init_chill(state)
                state
                    .options
                    .push(Entry::new(Command::Instruction(Call::Chill(bytes))));
            }
            // Call::Bond => self.init_bond(state),
            // Call::Unbond => self.init_unbond(state),
            // Call::ChangeRewardDestination => self.init_change_reward_destination(state),
            // Call::ChangeCommission => self.init_change_commission(state),
            // Call::KickNominators => self.init_kick_nominators(state),
            // Call::SetSessionKey => self.init_set_session_key(state),
            _ => {
                warn!("Unsupported call: {:?}", call);
                return;
            }
        }
    }

    // DEPRECATE
    fn init_confirmation(&self, state: &mut ListState, call: Call) {
        match call {
            // Call::Chill(_) => self.init_chill(state),
            // Call::Bond => self.init_bond(state),
            // Call::Unbond => self.init_unbond(state),
            // Call::ChangeRewardDestination => self.init_change_reward_destination(state),
            // Call::ChangeCommission => self.init_change_commission(state),
            // Call::KickNominators => self.init_kick_nominators(state),
            // Call::SetSessionKey => self.init_set_session_key(state),
            _ => {
                warn!("Unsupported call: {:?}", call);
                return;
            }
        }
    }

    // fn init_chill(&self, state: &mut ListState) {
    //     state
    //         .options
    //         .push(Entry::new(Command::Instruction(Call::Chill)));
    //     state
    //         .options
    //         .push(Entry::new(Command::Text("cancel".to_string())));
    // }

    pub fn move_down(&self) -> Option<Entry<Call>> {
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

    pub fn move_up(&self) -> Option<Entry<Call>> {
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

    pub fn show(&self) {
        self.on_init(Mode::Menu, None);
    }

    pub fn close(&self) {
        let mut state = self.state.write().unwrap();
        state.is_visible = false;
    }

    pub fn get_selected(&self) -> Option<Entry<Call>> {
        let state = self.state.read().unwrap();
        state.get_selected()
    }

    pub fn get_options_filtered(&self) -> Vec<Entry<Call>> {
        let state = self.state.read().unwrap();
        state.get_options_filtered()
    }

    pub fn get_selected_call(&self) -> Option<Call> {
        let state = self.state.read().unwrap();
        state.get_selected_call()
    }

    pub fn get_mode(&self) -> Mode {
        let state = self.state.read().unwrap();
        state.mode.clone()
    }

    pub fn show_transaction_status(&self) {
        self.on_init(Mode::Transaction, None);
    }

    pub fn update_transaction_status(&self, message: &str) {
        let mut state = self.state.write().unwrap();
        state.spinner_counter += 1;
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(message.to_string())));
    }

    pub fn show_chill_details(&self, bytes: Vec<u8>) {
        self.on_init(Mode::Details, Some(Call::Chill(bytes)));
    }

    // Input actions
    pub fn set_input_focus(&self) {
        let mut state = self.state.write().unwrap();
        state.input.set_focus();
    }

    pub fn clear_input_focus(&self) {
        let mut state = self.state.write().unwrap();
        state.input.clear_focus();
    }

    pub fn insert_input_char(&self, new_char: char) {
        let mut state = self.state.write().unwrap();
        state.input.insert_char(new_char);
    }

    pub fn delete_input_char(&self) {
        let mut state = self.state.write().unwrap();
        state.input.delete_char();
    }

    pub fn move_cursor_left(&self) {
        let mut state = self.state.write().unwrap();
        state.input.move_cursor_left();
    }

    pub fn move_cursor_right(&self) {
        let mut state = self.state.write().unwrap();
        state.input.move_cursor_right();
    }

    pub fn set_input_autocomplete(&self) {
        let mut state = self.state.write().unwrap();
        if let Some(call) = state.get_selected_call() {
            state.input.set_value(call.to_string());
        }
    }

    pub fn execute_with_password<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&str) -> Result<R, E>,
    {
        let state = self.state.read().unwrap();

        state.input.execute_with_password(f)
    }

    // pub fn execute_with_password(&self) -> String {
    //     let state = self.state.read().unwrap();
    //     state
    //         .input
    //         .execute_with_password(|password| password.to_string())
    // }
}

impl Widget for &PopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        if !state.is_visible {
            return; // Do not render if popup is not active.
        }

        match state.mode {
            Mode::Menu => render_menu(area, buf, &mut state),
            Mode::Details => render_details(area, buf, &mut state),
            Mode::Confirm => render_confirmation(area, buf, &mut state),
            Mode::Transaction => render_transaction(area, buf, &mut state),
        }
    }
}

fn render_menu(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    // Split the area into main body to show all options, and footer to show the input field
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Details
            Constraint::Length(3), // InputField as command mode (label + input)
        ])
        .split(area);

    let block = Block::new()
        .set_style(THEME.block.active)
        .padding(Padding::symmetric(0, 1));

    let options = state.get_options_filtered();
    let rows = options.iter().map(|e| e.to_row(state.mode.clone(), None));

    let widths = [
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Fill(2),
        Constraint::Length(2),
    ];

    let header_labels = vec!["", "command", "description", ""];

    let table = Table::new(rows, widths)
        .block(block)
        .header(Row::new(header_labels).set_style(THEME.table.header))
        .style(THEME.table.base)
        .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    // NOTE: ensure that the selected entry is always the first one on the list
    state.table_state.select(Some(0));

    StatefulWidget::render(table, area[0], buf, &mut state.table_state);

    let call = state.get_selected_call();

    // Render input area
    state.input.as_command(call).render(area[1], buf);
}

fn render_details(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    // Split the area into header to show transaction details and password input area
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Details
            Constraint::Length(3), // InputField as password mode (label + input)
        ])
        .split(area);

    // NOTE: Should only be one entry when rendering details
    if let Some(entry) = state.options.get(0) {
        let extrinsic = Line::from(vec![
            Span::styled(
                "extrinsic ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(format!("<{}>", entry.command())),
        ]);

        let call_data_label = Line::from(vec![
            Span::styled(
                "call data ",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("ctrl+y copy", Style::default().fg(Color::Yellow)),
        ]);
        let call_data = Line::from(entry.to_hex());

        let details = Paragraph::new(vec![extrinsic, call_data_label, call_data])
            .block(
                Block::new()
                    .padding(Padding::proportional(1))
                    .style(Style::default().bg(Color::Rgb(52, 50, 51))),
            )
            .wrap(Wrap { trim: false });
        details.render(area[0], buf);
    }

    // Render input area
    state.input.as_password().render(area[1], buf);
}

// DEPRECATED
fn _render_confirmation(area: Rect, buf: &mut Buffer, state: &mut ListState) {
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
        .style(THEME.table.base)
        .block(block)
        .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, area, buf, &mut state.table_state);
}

fn render_confirmation(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    // Split the area into header to show transaction details and password input area
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4), // Details
            Constraint::Min(3),    // Password (label + input)
        ])
        .split(area);

    // Define header details
    let details = Paragraph::new(vec![
        Line::from(format!("method: <chill>")).style(Style::default()),
        Line::from(format!("call data: <0x00..>")).style(Style::default()),
    ])
    .block(Block::bordered().title(" Confirm "));
    details.render(area[0], buf);

    // Render input area
    state.input.as_password().render(area[1], buf);
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
    let widths = [Constraint::Length(7), Constraint::Fill(1)];
    let table = Table::new(rows, widths)
        .style(THEME.table.base)
        .block(block);
    // .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, area, buf, &mut state.table_state);
}

// impl<T: AsChar + std::fmt::Display + ToDescription + ToPlaceholder + ToHex + Clone> Entry<T> {
//     pub fn to_row(&self, mode: Mode, msg: Option<&str>) -> Row<'_> {
//         let command = self.get_command();
//         match command {
//             Command::Instruction(c) => {
//                 let mut cols = Vec::new();

//                 // Add menu-specific formatting
//                 match mode {
//                     Mode::Menu => {
//                         cols.push("".to_string());
//                         // cols.push(c.as_char().to_string());
//                         cols.push(c.to_string());
//                         cols.push(c.description());
//                         cols.push("".to_string());
//                     }
//                     Mode::Confirm => {
//                         cols.push(c.to_string());
//                         cols.push(c.description());
//                     }
//                     _ => {}
//                 }

//                 Row::new(cols)
//             }
//             Command::Text(t) => match mode {
//                 Mode::Transaction => {
//                     let mut cols = Vec::new();

//                     cols.push(Cell::from(msg.unwrap_or("").to_string()));
//                     cols.push(Cell::from(
//                         Line::from(format!("[{t}]")).alignment(Alignment::Right),
//                     ));

//                     Row::new(cols)
//                 }
//                 _ => Row::new(vec![t.to_string()]),
//             },
//         }
//     }
// }
