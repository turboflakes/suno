use crate::theme::THEME;
use crate::widgets::{
    input_field::{InputFieldWidget, Metadata as InputFieldMetadata},
    spinner::Spinner,
};
use log::warn;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Position, Rect},
    text::{Line, Span},
    widgets::{
        Block, Cell, Clear, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget,
        Wrap,
    },
};
use sp_arithmetic::Perbill;
use std::sync::{Arc, RwLock};
use suno_config::SupportedRuntime;
use suno_primitives::{
    call::Call,
    entry::{Command, Entry, ToDescription},
    session::Keys,
    staking::Payee,
    validator::ValidatorStatus,
    Validator,
};
use unicode_width::UnicodeWidthStr;

/// Popup modes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Menu,
    Confirm,
    Locked,
    Transaction,
}

#[derive(Debug, Clone, Default)]
pub struct PopupWidget {
    pub state: Arc<RwLock<PopupState>>,
}

#[derive(Debug)]
pub struct PopupState {
    options: Vec<Entry<Call>>,
    table_state: TableState,
    is_visible: bool,
    mode: Mode,
    input: InputFieldWidget,
    spinner: Spinner,
}

impl Default for PopupState {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            table_state: TableState::default(),
            is_visible: false,
            mode: Mode::default(),
            input: InputFieldWidget::new(),
            spinner: Spinner::default(),
        }
    }
}

impl PopupState {
    pub fn set_lock(&mut self) {
        self.mode = Mode::Locked;
    }

    pub fn set_confirm(&mut self) {
        self.mode = Mode::Confirm;
    }

    pub fn get_input_cursor_position(&self) -> Option<Position> {
        self.input.get_cursor_position()
    }

    pub fn get_input_parsed_call(&self) -> Option<Call> {
        self.input.get_parsed_call()
    }

    pub fn get_options_filtered(&self) -> Vec<Entry<Call>> {
        let input_value = self.input.value();
        let input_command = match input_value.split_once(' ') {
            None => input_value.as_str(),
            Some((command, _)) => command,
        };

        let out = self
            .options
            .iter()
            .filter(|e| e.command().to_lowercase().starts_with(input_command))
            .cloned()
            .collect();

        out
    }

    pub fn get_selected(&self) -> Option<Entry<Call>> {
        match self.mode {
            Mode::Menu => {
                let options = self.get_options_filtered();
                if options.is_empty() {
                    return None;
                }
                self.table_state.selected().and_then(|i| {
                    // clamp: if the filter shrank the list, use the last item
                    let i = i.min(options.len() - 1);
                    options.get(i).cloned()
                })
            }
            _ => self.table_state.selected().map(|i| self.options[i].clone()),
        }
    }

    pub fn get_selected_call(&self) -> Option<Call> {
        if let Some(selected) = self.get_selected() {
            match selected.get_command() {
                Command::Instruction { call, .. } => Some(call),
                _ => None,
            }
        } else {
            None
        }
    }
}

type ActiveEra = u32;

impl PopupWidget {
    pub fn on_init(&self, mode: Mode, context: Option<(ActiveEra, Validator)>) {
        let mut state = self.state.write().unwrap();
        state.options.clear();
        match mode {
            Mode::Menu => self.init_menu(&mut state, context),
            Mode::Transaction => self.init_transaction(&mut state),
            _ => {}
        }
        state.mode = mode;

        // Select the first option.
        if !state.options.is_empty() {
            state.table_state.select(Some(0));
        }

        // Make popup visible.
        state.is_visible = true;
    }

    fn _on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    fn init_menu(&self, state: &mut PopupState, context: Option<(ActiveEra, Validator)>) {
        let Some((active_era, validator)) = context else {
            return;
        };

        let runtime = validator.runtime().asset_hub_runtime();

        // Reset the input field to command mode and set metadata.
        let unit = runtime.token_symbol();
        let decimals = runtime.token_decimals();
        let metadata = InputFieldMetadata::new(unit, decimals);
        state.input.reset_as_command(Some(metadata));

        match validator.status {
            ValidatorStatus::Waiting | ValidatorStatus::Unknown => {
                // NOTE: Bonding calls are only available if validator is waiting or has been chilled.
                state.options.push(Entry::new(Command::Instruction {
                    call: Call::Bond {
                        amount: 0,
                        payee: Payee::default(),
                        max: Some(validator.free_balance_extended(4)),
                    },
                    bytes: None,
                }));
                state.options.push(Entry::new(Command::Instruction {
                    call: Call::Validate {
                        commission: Perbill::from_percent(0),
                        blocked: false,
                    },
                    bytes: None,
                }));
            }
            _ => {
                if validator.free_balance() > 0 {
                    state.options.push(Entry::new(Command::Instruction {
                        call: Call::BondExtra {
                            amount: 0,
                            max: Some(validator.free_balance_extended(4)),
                        },
                        bytes: None,
                    }));
                }

                if validator.bounded() > 0 {
                    state.options.push(Entry::new(Command::Instruction {
                        call: Call::Unbond {
                            amount: 0,
                            max: Some(validator.bounded_extended(4)),
                        },
                        bytes: None,
                    }));
                }

                if validator.unlocking(active_era) > 0 {
                    state.options.push(Entry::new(Command::Instruction {
                        call: Call::Rebond {
                            amount: 0,
                            max: Some(validator.unlocking_extended(active_era, 4)),
                        },
                        bytes: None,
                    }));
                }

                if validator.unlocked(active_era) > 0 {
                    state.options.push(Entry::new(Command::Instruction {
                        call: Call::WithdrawUnbonded {
                            max: Some(validator.unlocked_extended(active_era, 4)),
                        },
                        bytes: None,
                    }));
                }

                state.options.push(Entry::new(Command::Instruction {
                    call: Call::SetPayee {
                        payee: Payee::default(),
                    },
                    bytes: None,
                }));

                state.options.push(Entry::new(Command::Instruction {
                    call: Call::Validate {
                        commission: Perbill::from_percent(0),
                        blocked: false,
                    },
                    bytes: None,
                }));

                state.options.push(Entry::new(Command::Instruction {
                    call: Call::Chill,
                    bytes: None,
                }));
            }
        }

        state.options.push(Entry::new(Command::Instruction {
            call: Call::SetSessionKeys {
                keys: Keys::default(),
            },
            bytes: None,
        }));
    }

    pub fn init_confirm_and_sign(
        &self,
        runtime: SupportedRuntime,
        spec_version: u32,
        proxy_identity: String,
        stash_identity: String,
        call: Call,
        bytes: Vec<u8>,
    ) {
        let mut state = self.state.write().unwrap();
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(runtime.as_str_long().to_string())));
        state
            .options
            .push(Entry::new(Command::Text(spec_version.to_string())));
        state
            .options
            .push(Entry::new(Command::Text(proxy_identity)));
        state
            .options
            .push(Entry::new(Command::Text(stash_identity)));
        // NOTE: Instruction with the previusly selcted call and respective call_data
        state.options.push(Entry::new(Command::Instruction {
            call,
            bytes: Some(bytes),
        }));
        // NOTE: Rather than having a specific field to hold the call data bytes,
        // we just select the option in position 4th which is where it is being added.
        // Makes it easier to retrieve the selected option later to copy it to the clipboard;
        state.table_state.select(Some(4));
        // Change popup mode to confirmation mode
        state.mode = Mode::Confirm;
        // Reset the input field as a password field
        state.input.reset_as_password();
    }

    fn init_transaction(&self, state: &mut PopupState) {
        state.spinner.increment();
        state.options.push(Entry::new(Command::Text(
            "processing transaction".to_string(),
        )));
    }

    pub fn move_down(&self) -> Option<Entry<Call>> {
        let mut state = self.state.write().unwrap();
        let options = state.get_options_filtered();
        if options.is_empty() {
            return None;
        }
        if let Some(selected) = state.table_state.selected() {
            if selected == options.len() - 1 {
                state.table_state.select_first();
            } else {
                state.table_state.select(Some(selected + 1));
            }
            state.table_state.selected().map(|i| options[i].clone())
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Entry<Call>> {
        let mut state = self.state.write().unwrap();
        let options = state.get_options_filtered();
        if options.is_empty() {
            return None;
        }
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = options.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.select(Some(selected - 1));
            }

            state.table_state.selected().map(|i| options[i].clone())
        } else {
            None
        }
    }

    pub fn is_visible(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_visible
    }

    pub fn is_menu_visible(&self) -> bool {
        let state = self.state.read().unwrap();
        matches!(state.mode, Mode::Menu)
    }

    pub fn is_transaction_visible(&self) -> bool {
        let state = self.state.read().unwrap();
        matches!(state.mode, Mode::Transaction)
    }

    pub fn show_extrinsics(&self, active_era: ActiveEra, validator: Validator) {
        self.on_init(Mode::Menu, Some((active_era, validator)));
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
        self.on_init(Mode::Transaction, None);
    }

    pub fn update_transaction_status(&self, message: &str) {
        let mut state = self.state.write().unwrap();
        state.spinner.increment();
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(message.to_string())));
    }

    // Input actions
    pub fn set_input_focus(&self) -> bool {
        let mut state = self.state.write().unwrap();
        state.input.set_focus()
    }

    pub fn clear_input_focus(&self) {
        let mut state = self.state.write().unwrap();
        state.input.clear_focus();
    }

    pub fn set_lock_mode(&self) {
        let mut state = self.state.write().unwrap();
        state.set_lock();
        state.input.lock_input();
    }

    pub fn set_confirm_mode(&self) {
        let mut state = self.state.write().unwrap();
        state.set_confirm();
    }

    pub fn invalidate_input(&self, msg: &str) -> bool {
        let mut state = self.state.write().unwrap();
        state.input.invalidate(msg)
    }

    pub fn insert_input_char(&self, new_char: char) {
        let mut state = self.state.write().unwrap();
        state.input.insert_char(new_char);
    }

    pub fn delete_input_char(&self) {
        let mut state = self.state.write().unwrap();
        state.input.delete_char();

        if state.mode == Mode::Menu {
            let options = state.get_options_filtered();
            if options.is_empty() {
                return;
            }
            // NOTE: ensure to select the first entry as soon as options are not filtered out
            state.table_state.select(Some(0));
        }
    }

    pub fn insert_input_paste_data(&self, data: String) {
        let mut state = self.state.write().unwrap();
        state.input.paste_data(data);
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
            Mode::Confirm | Mode::Locked => render_confirm_and_sign(area, buf, &mut state),
            Mode::Transaction => render_transaction(area, buf, &mut state),
        }
    }
}

fn render_menu(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let block = Block::new()
        .style(THEME.block.active)
        .padding(Padding::symmetric(0, 1));

    let options = state.get_options_filtered();

    let rows = state.options.iter().map(|f| {
        let command = f.get_command();
        to_row(command, state.mode.clone(), None)
    });

    // Split the area into top header to show all options, a small central box to show the input field
    // and a bottom footer to show the error message
    let top_len = (options.len() as u16 + 3).clamp(4, 10);
    let [details_area, input_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(top_len), // Top header
            Constraint::Length(5),    // InputField as command mode (input (3) + invalid msg (2))
        ])
        .flex(Flex::End)
        .areas(area);

    // NOTE: Clear top header background and skip the inputfield, since is better
    // to be managed in the input widget
    Clear.render(details_area, buf);

    let widths = [
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Fill(2),
        Constraint::Length(2),
    ];

    let header_labels = vec!["", "extrinsic", "description", ""];

    let table = Table::new(rows, widths)
        .block(block)
        .header(Row::new(header_labels).style(THEME.table.header))
        .style(THEME.table.base)
        .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, details_area, buf, &mut state.table_state);

    // Render input area
    let call = state.get_selected_call();
    state.input.as_command(call).render(input_area, buf);
}

fn render_confirm_and_sign(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let block = Block::new()
        .style(THEME.block.active)
        .padding(Padding::proportional(1));

    // Get all required data from 'state.options' based on the indices established
    // in `init_menu`.
    let Some(network_entry) = state.options.first() else {
        return;
    };

    let Some(spec_version_entry) = state.options.get(1) else {
        return;
    };
    let Some(proxy_identity_entry) = state.options.get(2) else {
        return;
    };
    let Some(stash_identity_entry) = state.options.get(3) else {
        return;
    };
    let Some(call_entry) = state.options.get(4) else {
        return;
    };

    let network = Line::from(vec![Span::styled(
        format!(
            "{} ({})",
            network_entry.command(),
            spec_version_entry.command()
        ),
        THEME.paragraph.header_active,
    )])
    .alignment(Alignment::Right);

    let stash = Line::from(vec![
        Span::styled("stash ", THEME.paragraph.label),
        Span::raw(stash_identity_entry.command()),
    ]);

    let method = Line::from(vec![
        Span::styled("method ", THEME.paragraph.label),
        Span::raw(call_entry.to_method()),
    ]);
    let method_lines = calculate_text_wrapped_lines(&call_entry.to_method(), area.width);

    let proxy = Line::from(vec![
        Span::styled("proxy account ", THEME.paragraph.label),
        Span::raw(proxy_identity_entry.command()),
    ]);

    // Calculate spaces needed to show the `ctrl+shift+c copy on the right`
    let available_width = area.width.saturating_sub(4);
    let left_text = "call data";
    let spaces = available_width.saturating_sub((left_text.len() + 17) as u16);

    let call_data_label = Line::from(vec![
        Span::styled(left_text, THEME.paragraph.label),
        Span::raw(" ".repeat(spaces as usize)),
        Span::styled("ctrl+shift+c", THEME.paragraph.base),
        Span::raw(" "),
        Span::styled("copy", THEME.paragraph.label),
    ]);

    let call_data = Line::from(call_entry.to_hex());
    let call_data_lines = calculate_text_wrapped_lines(&call_entry.to_hex(), area.width);

    // Split the area into header to show transaction details and password input area
    let [details_area, input_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(8 + method_lines + call_data_lines), // Details
            Constraint::Length(5), // InputField as password mode (input (3) + invalid msg (2))
        ])
        .flex(Flex::End)
        .areas(area);

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

    Clear.render(details_area, buf);

    details.render(details_area, buf);

    // Render input area
    state.input.as_password().render(input_area, buf);
}

fn render_transaction(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let horizontal = Layout::horizontal([Constraint::Max(56)]);
    let [area] = horizontal.areas(area);

    Clear.render(area, buf);

    let block = Block::new()
        .style(THEME.block.main)
        .padding(Padding::proportional(1));

    let spinner_progress = state.spinner.frame();

    let rows = state.options.iter().map(|f| {
        let command = f.get_command();
        to_row(command, state.mode.clone(), Some(spinner_progress))
    });
    let widths = [Constraint::Fill(1), Constraint::Length(7)];
    let table = Table::new(rows, widths)
        .style(THEME.table.base)
        .block(block);
    // .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, area, buf, &mut state.table_state);
}

pub fn to_row(command: Command<Call>, mode: Mode, msg: Option<&str>) -> Row<'_> {
    match command {
        Command::Instruction { call, .. } => {
            let mut cols = Vec::new();

            // Add menu-specific formatting
            if mode == Mode::Menu {
                cols.push("".to_string());
                cols.push(format!("/{}", call));
                cols.push(call.description());
                cols.push("".to_string());
            }

            Row::new(cols)
        }
        Command::Text(t) => match mode {
            Mode::Transaction => {
                let cols = vec![
                    Cell::from(Line::from(t.to_string())),
                    Cell::from(
                        Line::from(msg.unwrap_or("").to_string()).alignment(Alignment::Right),
                    ),
                ];

                Row::new(cols)
            }
            _ => Row::new(vec![t.to_string()]),
        },
        // _ => Row::new(vec!["".to_string()]),
    }
}

fn calculate_text_wrapped_lines(text: &str, area_width: u16) -> u16 {
    let mut total_lines = 0;

    for line in text.lines() {
        let line_width = line.width();
        let area_width = area_width as usize;

        if line_width == 0 {
            total_lines += 1;
        } else {
            // Calculate how many lines this single line will wrap into
            let wrapped = line_width.div_ceil(area_width);
            total_lines += wrapped as u16;
        }
    }

    total_lines.max(1)
}
