use crate::call::{Call, CallError};
use crate::entry::{
    AsBytes, AsChar, Command, Entry, ToDescription, ToHex, ToJson, ToMethod, ToPlaceholder,
};
use crate::theme::THEME;
use crate::widgets::input_field::InputFieldWidget;
use log::{info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Position, Rect},
    style::{Color, Modifier, Style, Styled},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Cell, Padding, Paragraph, Row, StatefulWidget, Table,
        TableState, Widget, Wrap,
    },
};
use std::sync::{Arc, RwLock};
use std::time::Instant;
use suno_config::SupportedRuntime;
use suno_primitives::{staking::Payee, tx::Bytes};

/// Popup modes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Menu,
    ConfirmAndSign,
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

    pub fn get_input_parsed_call(&self) -> Option<Call> {
        self.input.get_parsed_call()
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
    pub fn on_init(&self, mode: Mode) {
        let mut state = self.state.write().unwrap();
        state.options.clear();
        match mode {
            Mode::Menu => self.init_menu(&mut state),
            Mode::Transaction => self.init_transaction(&mut state),
            _ => {}
        }
        state.mode = mode;

        // Select the first option.
        if !state.options.is_empty() {
            state.table_state.select(Some(0));
        }

        // Reset the input field to command mode.
        state.input.reset_as_command();
        // Make popup visible.
        state.is_visible = true;
    }

    fn on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    fn init_menu(&self, state: &mut ListState) {
        // TODO: Define supported calls depending on the context
        state
            .options
            .push(Entry::new(Command::Instruction(Call::Bond {
                amount: 0,
                payee: Payee::default(),
            })));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::BondExtra {
                amount: 0,
            })));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::ChangeCommission)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::ChangePayee)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::Chill)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::KickNominators)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::SetSessionKey)));
        state
            .options
            .push(Entry::new(Command::Instruction(Call::Unbond { amount: 0 })));
    }

    fn init_transaction(&self, state: &mut ListState) {
        state.spinner_start_time = Instant::now();
        state.spinner_counter = 0;
        state.options.push(Entry::new(Command::Text(
            "processing transaction".to_string(),
        )));
    }

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

    pub fn show_extrinsics(&self) {
        self.on_init(Mode::Menu);
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

    pub fn get_input_parsed_call(&self) -> Option<Call> {
        let state = self.state.read().unwrap();
        state.get_input_parsed_call()
    }

    pub fn get_mode(&self) -> Mode {
        let state = self.state.read().unwrap();
        state.mode.clone()
    }

    pub fn show_transaction_status(&self) {
        self.on_init(Mode::Transaction);
    }

    pub fn update_transaction_status(&self, message: &str) {
        let mut state = self.state.write().unwrap();
        state.spinner_counter += 1;
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(message.to_string())));
    }

    pub fn confirm_and_sign(
        &self,
        runtime: &SupportedRuntime,
        spec_version: u32,
        proxy_identity: String,
        stash_identity: String,
        call: Call,
        bytes: Bytes,
    ) {
        let mut state = self.state.write().unwrap();
        state.mode = Mode::ConfirmAndSign;
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(runtime.to_string())));
        state
            .options
            .push(Entry::new(Command::Text(spec_version.to_string())));
        state
            .options
            .push(Entry::new(Command::Text(proxy_identity)));
        state
            .options
            .push(Entry::new(Command::Text(stash_identity)));
        state.options.push(Entry::new(Command::Instruction(call)));
        state.options.push(Entry::new(Command::Bytes(bytes)));
        // NOTE: Rather than having a specific field to hold the call data bytes,
        // we just select the option in position the 5th which is where it was added.
        // Makes it easier to retrieve the selected option later on to copy it to clipboard;
        state.table_state.select(Some(5));
        // Reset the input field as a password field
        state.input.reset_as_password();
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
}

impl Widget for &PopupWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        if !state.is_visible {
            return; // Do not render if popup is not active.
        }

        match state.mode {
            Mode::Menu => render_menu(area, buf, &mut state),
            Mode::ConfirmAndSign => render_confirm_and_sign(area, buf, &mut state),
            Mode::Transaction => render_transaction(area, buf, &mut state),
        }
    }
}

fn render_menu(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    let block = Block::new()
        .set_style(THEME.block.active)
        .padding(Padding::symmetric(0, 1));

    let options = state.get_options_filtered();
    let rows = options.iter().map(|e| e.to_row(state.mode.clone(), None));

    // Split the area into top header to show all options, a small central box to show the input field
    // and a bottom footer to show the error message
    let top_len = (options.len() as u16 + 3).clamp(4, 10);
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(top_len), // Top header
            Constraint::Length(5),    // InputField as command mode (label + input)
        ])
        .flex(Flex::End)
        .split(area);

    let widths = [
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Fill(2),
        Constraint::Length(2),
    ];

    let header_labels = vec!["", "extrinsic", "description", ""];

    let table = Table::new(rows, widths)
        .block(block)
        .header(Row::new(header_labels).set_style(THEME.table.header))
        .style(THEME.table.base)
        .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    // NOTE: ensure that the selected entry is always the first one on the list
    state.table_state.select(Some(0));

    StatefulWidget::render(table, area[0], buf, &mut state.table_state);

    // Render input area
    let call = state.get_selected_call();
    state.input.as_command(call).render(area[1], buf);
}

fn render_confirm_and_sign(area: Rect, buf: &mut Buffer, state: &mut ListState) {
    let block = Block::new()
        .style(THEME.block.active)
        .padding(Padding::proportional(1));

    // Split the area into header to show transaction details and password input area
    let area = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(6),    // Details
            Constraint::Length(3), // InputField as password mode
        ])
        .split(area);

    // Get network from the first position in the list
    let Some(network_entry) = state.options.get(0) else {
        return;
    };
    // Get spec_version from the second position in the list
    let Some(spec_version_entry) = state.options.get(1) else {
        return;
    };
    // Get proxy identity from the third position in the list
    let Some(proxy_identity_entry) = state.options.get(2) else {
        return;
    };
    // Get proxied identity from the third position in the list
    let Some(stash_identity_entry) = state.options.get(3) else {
        return;
    };
    // Get call from the fourth position in the list
    let Some(call_entry) = state.options.get(4) else {
        return;
    };
    // Get bytes from the fifth position in the list
    let Some(bytes_entry) = state.options.get(5) else {
        return;
    };

    let network = Line::from(vec![
        Span::styled("chain ", THEME.paragraph.header_active),
        Span::raw(format!(
            "{} ({})",
            network_entry.command(),
            spec_version_entry.command()
        )),
    ])
    .alignment(Alignment::Right);

    let stash = Line::from(vec![
        Span::styled("stash ", THEME.paragraph.header),
        Span::raw(stash_identity_entry.command()),
    ]);

    let method = Line::from(vec![
        Span::styled("method ", THEME.paragraph.header),
        Span::raw(format!("{}", call_entry.to_method())),
    ]);

    let proxy = Line::from(vec![
        Span::styled("proxy account ", THEME.paragraph.header),
        Span::raw(proxy_identity_entry.command()),
    ]);

    // Calculate spaces needed to show the `ctrl+shift+c copy on the right`
    let available_width = area[0].width.saturating_sub(4);
    let left_text = "call data";
    let spaces = available_width.saturating_sub((left_text.len() + 17) as u16);

    let call_data_label = Line::from(vec![
        Span::styled(left_text, THEME.paragraph.header),
        Span::raw(" ".repeat(spaces as usize)),
        Span::styled("ctrl+shift+c", THEME.paragraph.base),
        Span::raw(" "),
        Span::styled("copy", THEME.paragraph.label),
    ]);

    let call_data = Line::from(bytes_entry.to_hex());

    let details = Paragraph::new(vec![
        network,
        stash,
        method,
        proxy,
        call_data_label,
        call_data,
    ])
    .block(block)
    .wrap(Wrap { trim: false });
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

impl<
        T: std::fmt::Display
            + ToDescription
            + ToPlaceholder
            + ToJson
            + ToMethod
            + ToHex
            + AsBytes
            + Clone,
    > Entry<T>
{
    pub fn to_row(&self, mode: Mode, msg: Option<&str>) -> Row<'_> {
        let command = self.get_command();
        match command {
            Command::Instruction(c) => {
                let mut cols = Vec::new();

                // Add menu-specific formatting
                match mode {
                    Mode::Menu => {
                        cols.push("".to_string());
                        cols.push(format!("/{}", c.to_string()));
                        cols.push(c.description());
                        cols.push("".to_string());
                    }
                    _ => {}
                }

                Row::new(cols)
            }
            Command::Text(t) => match mode {
                Mode::Transaction => {
                    let mut cols = Vec::new();
                    cols.push(Cell::from(msg.unwrap_or("").to_string()));
                    cols.push(Cell::from(
                        Line::from(format!("[{t}]")).alignment(Alignment::Right),
                    ));

                    Row::new(cols)
                }
                _ => Row::new(vec![t.to_string()]),
            },
            _ => Row::new(vec!["".to_string()]),
        }
    }
}
