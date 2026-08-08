use crate::widgets::{
    input_field::{InputFieldWidget, Metadata as InputFieldMetadata},
    spinner::Spinner,
};
use image::DynamicImage;
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Position, Rect},
    text::{Line, Span},
    widgets::{
        Block, Cell, Clear, Padding, Paragraph, Row, StatefulWidget, Table, TableState, Widget,
        Wrap,
    },
};
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};
use sp_arithmetic::Perbill;
use std::sync::{Arc, RwLock};
use suno_actions::{ChainSpecsContext, ConfirmationContext, MetadataContext, ThreadAction};
use suno_config::{SupportedRuntime, CONFIG};
use suno_primitives::{
    call::Call,
    entry::{Command, Entry, ToDescription},
    session::{Keys, Proof},
    staking::Payee,
    Chain, Validator,
};
use suno_qrcode::{MetadataState, MetadataWidget, QrCodeWidget, ScannerWidget};
use tokio::sync::mpsc::UnboundedSender;
use tracing::warn;
use unicode_width::UnicodeWidthStr;

#[derive(Clone, PartialEq, Eq)]
struct ValidatorContext {
    era: ActiveEra,
    validator: Validator,
}

#[derive(Clone, PartialEq, Eq)]
struct ChainContext {
    runtime: SupportedRuntime,
}

/// Context holds the initialization data for a popup. Carries the heavy, mode-specific
/// payload only while building the popup; it is not stored in state.
enum Context {
    ValidatorMenu(Box<ValidatorContext>),
    ChainMenu(Box<ChainContext>),
    Confirmation(Box<ConfirmationContext>),
    Transaction,
    Update,
    ChainSpecs(Box<ChainSpecsContext>),
    Metadata(Box<MetadataContext>),
}

impl Context {
    fn mode(&self) -> Mode {
        match self {
            Context::ValidatorMenu(_) => Mode::Menu,
            Context::ChainMenu(_) => Mode::Menu,
            Context::Confirmation(_) => Mode::Confirmation,
            Context::Transaction => Mode::Transaction,
            Context::Update => Mode::Update,
            Context::ChainSpecs(_) => Mode::ChainSpecs,
            Context::Metadata(_) => Mode::Metadata,
        }
    }
}

/// Popup status_modes.
#[derive(Clone, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Hidden,
    Menu,
    Confirmation,
    Transaction,
    Update,
    ChainSpecs,
    Metadata,
}

#[derive(Clone)]
pub struct PopupWidget {
    pub state: Arc<RwLock<PopupState>>,
    picker: Arc<Picker>,
}

impl Default for PopupWidget {
    fn default() -> Self {
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        Self {
            state: Arc::default(),
            picker: Arc::new(picker),
        }
    }
}

/// Per-session scanner state. Exists only while the popup is showing the scanner.
/// Dropping this (e.g. on `close`) drops `_ctrl`, which disconnects the scanner
/// thread's receiver and stops it.
pub struct ScannerSession {
    _ctrl: UnboundedSender<ThreadAction>,
    _picker: Arc<Picker>,
    frame_protocol: Option<StatefulProtocol>,
}

impl ScannerSession {
    pub fn new(ctrl: UnboundedSender<ThreadAction>, picker: Arc<Picker>) -> Self {
        Self {
            _ctrl: ctrl,
            _picker: picker,
            frame_protocol: None,
        }
    }

    pub fn set_frame(&mut self, frame: DynamicImage) {
        self.frame_protocol = Some(self._picker.new_resize_protocol(frame));
    }
}

pub struct PopupState {
    options: Vec<Entry<Call>>,
    table_state: TableState,
    mode: Mode,
    input: InputFieldWidget,
    spinner: Spinner,
    title: Option<String>,
    label: Option<String>,
    scanner: Option<ScannerSession>,
    masked: bool,
    metadata: Option<MetadataState>,
}

impl Default for PopupState {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            table_state: TableState::default(),
            mode: Mode::default(),
            input: InputFieldWidget::new(),
            spinner: Spinner::default(),
            title: None,
            label: None,
            scanner: None,
            masked: true,
            metadata: None,
        }
    }
}

impl PopupState {
    fn is_hidden(&self) -> bool {
        matches!(self.mode, Mode::Hidden)
    }

    fn is_visible(&self) -> bool {
        !self.is_hidden()
    }

    fn is_masked(&self) -> bool {
        self.masked
    }

    pub fn get_input_cursor_position(&self) -> Option<Position> {
        self.input.get_cursor_position()
    }

    pub fn get_input_parsed_call(&self) -> Option<Call> {
        self.input.get_parsed_call()
    }

    pub fn get_options_filtered(&self) -> Vec<Entry<Call>> {
        let input_value = self.input.raw_value();
        let input_command = match input_value.split_once(' ') {
            None => input_value.as_str(),
            Some((command, _)) => command,
        };

        self.options
            .iter()
            .filter(|e| e.command().to_lowercase().starts_with(input_command))
            .cloned()
            .collect()
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
    fn on_init(&self, context: Context) {
        let mut state = self.state.write().unwrap();
        state.options.clear();

        match &context {
            Context::ValidatorMenu(ctx) => self.init_validator_menu(&mut state, ctx),
            Context::ChainMenu(ctx) => self.init_chain_menu(&mut state, ctx),
            Context::Confirmation(ctx) => self.init_confirmation(&mut state, ctx),
            Context::Transaction => self.init_transaction(&mut state),
            Context::Update => self.init_update(&mut state),
            Context::ChainSpecs(ctx) => self.init_chain_specs_qrcode(&mut state, ctx),
            Context::Metadata(ctx) => self.init_metadata_qrcode(&mut state, ctx),
        }

        state.mode = context.mode();
    }

    fn _on_err(&self, err: Box<dyn std::error::Error>) {
        warn!("Failed with error: {}", err);
        // TODO: Set chain state to error
    }

    fn init_validator_menu(&self, state: &mut PopupState, ctx: &ValidatorContext) {
        // If the validator is not a proxy or has no commands, do not show the menu.
        if !ctx.validator.is_proxy_valid() && !ctx.validator.is_commands_available() {
            return;
        }

        // Set pop-up title as the validator selected
        state.title = Some(ctx.validator.display_identity());

        let runtime = ctx.validator.runtime().asset_hub_runtime();

        // Reset the input field to command mode and set metadata.
        let unit = runtime.token_symbol();
        let decimals = runtime.token_decimals();
        let metadata = InputFieldMetadata::new()
            .with_unit(unit)
            .with_decimals(decimals)
            .with_custom_commands(ctx.validator.commands.clone());
        state.input.reset_as_command(Some(metadata));

        // For each supported proxy, push the respective calls depending on the validator's status.
        ctx.validator.proxies.iter().for_each(|p| {
            // NOTE: Bonding calls are only available if validator status is unknown.
            let bond = Call::Bond {
                amount: 0,
                payee: Payee::default(),
                max: Some(ctx.validator.free_balance_extended(4)),
            };
            if p.proxy().can_call(&bond) && ctx.validator.is_unknown() {
                state.options.push(Entry::new(Command::Instruction {
                    call: bond,
                    bytes: None,
                }));
            }

            let bond_extra = Call::BondExtra {
                amount: 0,
                max: Some(ctx.validator.free_balance_extended(4)),
            };
            if p.proxy().can_call(&bond_extra)
                && ctx.validator.is_active_or_waiting()
                && ctx.validator.free_balance() > 0
            {
                state.options.push(Entry::new(Command::Instruction {
                    call: bond_extra,
                    bytes: None,
                }));
            }

            let unbond = Call::Unbond {
                amount: 0,
                max: Some(ctx.validator.bounded_extended(4)),
            };
            if p.proxy().can_call(&unbond)
                && ctx.validator.is_active_or_waiting()
                && ctx.validator.bounded() > 0
            {
                state.options.push(Entry::new(Command::Instruction {
                    call: unbond,
                    bytes: None,
                }));
            }

            let rebond = Call::Rebond {
                amount: 0,
                max: Some(ctx.validator.unlocking_extended(ctx.era, 4)),
            };
            if p.proxy().can_call(&rebond)
                && ctx.validator.is_active_or_waiting()
                && ctx.validator.unlocking(ctx.era) > 0
            {
                state.options.push(Entry::new(Command::Instruction {
                    call: rebond,
                    bytes: None,
                }));
            }

            let withdraw = Call::WithdrawUnbonded {
                max: Some(ctx.validator.unlocked_extended(ctx.era, 4)),
            };
            if p.proxy().can_call(&withdraw)
                && ctx.validator.is_active_or_waiting()
                && ctx.validator.unlocked(ctx.era) > 0
            {
                state.options.push(Entry::new(Command::Instruction {
                    call: withdraw,
                    bytes: None,
                }));
            }

            let set_payee = Call::SetPayee {
                payee: Payee::default(),
            };
            if p.proxy().can_call(&set_payee) && ctx.validator.is_active_or_waiting() {
                state.options.push(Entry::new(Command::Instruction {
                    call: set_payee,
                    bytes: None,
                }));
            }

            // NOTE: Validate calls are always available.
            let validate = Call::Validate {
                commission: Perbill::from_percent(0),
                blocked: false,
            };
            if p.proxy().can_call(&validate) {
                state.options.push(Entry::new(Command::Instruction {
                    call: validate,
                    bytes: None,
                }));
            }

            let chill = Call::Chill;
            if p.proxy().can_call(&chill) && ctx.validator.is_active_or_waiting() {
                state.options.push(Entry::new(Command::Instruction {
                    call: chill,
                    bytes: None,
                }));
            }

            let set_keys = Call::SetKeys {
                keys: Keys::default(),
                proof: Proof::default(),
            };
            if p.proxy().can_call(&set_keys) && ctx.validator.is_active_or_waiting() {
                state.options.push(Entry::new(Command::Instruction {
                    call: set_keys,
                    bytes: None,
                }));
            }

            let purge_keys = Call::PurgeKeys;
            if p.proxy().can_call(&purge_keys)
                && ctx.validator.is_active_or_waiting()
                && ctx.validator.has_keys()
            {
                state.options.push(Entry::new(Command::Instruction {
                    call: purge_keys,
                    bytes: None,
                }));
            }
        });

        // Set pop-up label as configured host, if custom commands are defined
        if !ctx.validator.commands.is_empty() {
            state.label = Some(ctx.validator.host(state.is_masked()));
        }

        // For each custom commands, push the respective calls depending on the validator's status.
        ctx.validator.commands.iter().for_each(|c| {
            state.options.push(Entry::new(Command::Instruction {
                call: Call::Custom(c.clone()),
                bytes: None,
            }));
        });

        // Select the first option.
        if !state.options.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn init_chain_menu(&self, state: &mut PopupState, ctx: &ChainContext) {
        // Reset the input field to command mode. and set metadata.
        state.input.reset_as_command(None);

        // Set pop-up title as the chain selected
        state.title = Some(format!(
            "{} NETWORK",
            ctx.runtime.to_string().to_uppercase()
        ));

        state.options.push(Entry::new(Command::Instruction {
            call: Call::ChainSpecs {
                chain_name: ctx.runtime.to_string(),
            },
            bytes: None,
        }));

        state.options.push(Entry::new(Command::Instruction {
            call: Call::Metadata {
                chain_name: ctx.runtime.to_string(),
            },
            bytes: None,
        }));

        // Select the first option.
        if !state.options.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    fn init_confirmation(&self, state: &mut PopupState, ctx: &ConfirmationContext) {
        state.options.push(Entry::new(Command::Text(
            ctx.runtime.as_str_long().to_string(),
        )));
        state
            .options
            .push(Entry::new(Command::Text(ctx.spec_version.to_string())));
        state
            .options
            .push(Entry::new(Command::Text(ctx.proxy_identity.clone())));
        state
            .options
            .push(Entry::new(Command::Text(ctx.stash_identity.clone())));

        // Instruction with the previously selected call and respective call_data_bytes
        state.options.push(Entry::new(Command::Instruction {
            call: ctx.call.clone(),
            bytes: Some(ctx.call_data_bytes.clone()),
        }));

        // NOTE: Rather than having a specific field to hold the call data bytes,
        // we just select the option in position 4th which is where it is being added.
        // Makes it easier to retrieve the selected option later to copy it to the clipboard;
        state.table_state.select(Some(4));

        // NOTE: Make QR Data available only if qrcode signing is enabled
        if ctx.runtime.is_qrcode_enabled() {
            state
                .options
                .push(Entry::new(Command::Data(ctx.qr_bytes.clone())));
        }

        // Reset the input field as a password field
        state.input.reset_as_password();
    }

    fn init_transaction(&self, state: &mut PopupState) {
        state.spinner.increment();
        state.options.push(Entry::new(Command::Text(
            "processing transaction".to_string(),
        )));
    }

    fn init_update(&self, state: &mut PopupState) {
        state.spinner.increment();
        state
            .options
            .push(Entry::new(Command::Text("starting update".to_string())));
    }

    fn init_chain_specs_qrcode(&self, state: &mut PopupState, ctx: &ChainSpecsContext) {
        state.title = Some(ctx.runtime.as_str_long().to_string());
        state.label = Some("chain-specs".to_string());

        // Legacy name
        state
            .options
            .push(Entry::new(Command::Text(ctx.runtime.legacy_name())));
        // Chain spec version
        state
            .options
            .push(Entry::new(Command::Text(ctx.spec_version.to_string())));
        // Genesis hash
        state.options.push(Entry::new(Command::Data(
            ctx.runtime.chain_genesis_hash().0.into(),
        )));
        // Address prefix
        state.options.push(Entry::new(Command::Text(
            ctx.runtime.account_format().to_string(),
        )));
        // Unit
        state.options.push(Entry::new(Command::Text(
            ctx.runtime.token_symbol().to_string(),
        )));
        // QR Data
        state
            .options
            .push(Entry::new(Command::Data(ctx.qr_bytes.clone())));
    }

    fn init_metadata_qrcode(&self, state: &mut PopupState, ctx: &MetadataContext) {
        state.title = Some(ctx.runtime.as_str_long().to_string());
        state.label = Some("metadata".to_string());

        // Legacy name
        state
            .options
            .push(Entry::new(Command::Text(ctx.runtime.legacy_name())));
        // Chain spec version
        state
            .options
            .push(Entry::new(Command::Text(ctx.spec_version.to_string())));
        // Genesis hash
        state.options.push(Entry::new(Command::Data(
            ctx.runtime.chain_genesis_hash().0.into(),
        )));

        // Set Current Metadata QR Data
        state.metadata = Some(MetadataState::new(&ctx.qr_bytes));
    }

    pub fn show_validator_commands(&self, validator: &Validator, active_era: ActiveEra) {
        let ctx = ValidatorContext {
            era: active_era,
            validator: validator.clone(),
        };
        let ctx = Context::ValidatorMenu(Box::new(ctx));
        self.on_init(ctx);
    }

    pub fn show_chain_commands(&self, chain: &Chain) {
        let ctx = ChainContext {
            runtime: chain.runtime(),
        };
        let ctx = Context::ChainMenu(Box::new(ctx));
        self.on_init(ctx);
    }

    pub fn show_confirm_and_sign(&self, ctx: &ConfirmationContext) {
        let ctx = Context::Confirmation(Box::new(ctx.clone()));
        self.on_init(ctx);
    }

    pub fn show_chain_specs_qrcode(&self, ctx: &ChainSpecsContext) {
        let ctx = Context::ChainSpecs(Box::new(ctx.clone()));
        self.on_init(ctx);
    }

    pub fn show_metadata_qrcode(&self, ctx: &MetadataContext) {
        let ctx = Context::Metadata(Box::new(ctx.clone()));
        self.on_init(ctx);
    }

    pub fn show_transaction_status(&self) {
        self.on_init(Context::Transaction);
    }

    pub fn show_update_status(&self) {
        self.on_init(Context::Update);
    }

    pub fn advance_metadata_frame(&self) {
        let mut state = self.state.write().unwrap();
        if let Some(m) = state.metadata.as_mut() {
            m.advance_frame()
        }
    }

    pub fn is_hidden(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_hidden()
    }

    pub fn is_visible(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_visible()
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

    pub fn close(&self) {
        let mut state = self.state.write().unwrap();
        state.mode = Mode::Hidden;
        state.scanner = None;
        state.title = None;
        state.label = None;
        state.metadata = None;
        state.input.clear_focus();
    }

    pub fn start_scanner(&self, ctrl: UnboundedSender<ThreadAction>) {
        let mut state = self.state.write().unwrap();
        state.scanner = Some(ScannerSession::new(ctrl, Arc::clone(&self.picker)));
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

    pub fn is_confirmation_mode(&self) -> bool {
        let state = self.state.read().unwrap();
        matches!(state.mode, Mode::Confirmation)
    }

    pub fn is_menu_mode(&self) -> bool {
        let state = self.state.read().unwrap();
        matches!(state.mode, Mode::Menu)
    }

    pub fn can_close(&self) -> bool {
        let state = self.state.read().unwrap();
        matches!(
            state.mode,
            Mode::Menu | Mode::ChainSpecs | Mode::Metadata | Mode::Confirmation
        )
    }

    pub fn is_masked(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_masked()
    }

    pub fn toggle_mask(&self) {
        let mut state = self.state.write().unwrap();
        state.masked = !state.is_masked();
    }

    pub fn update_transaction_status(&self, message: &str) {
        let mut state = self.state.write().unwrap();
        state.spinner.increment();
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(message.to_string())));
    }

    pub fn change_update_status(&self, message: &str) {
        let mut state = self.state.write().unwrap();
        state.spinner.increment();
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(message.to_string())));
    }

    pub fn show_upgrade_complete(&self, message: &str) {
        let mut state = self.state.write().unwrap();
        state.spinner.complete();
        state.options.clear();
        state
            .options
            .push(Entry::new(Command::Text(message.to_string())));
    }

    pub fn show_upgrade_error(&self) {
        let mut state = self.state.write().unwrap();
        state.spinner.error();
        state.options.clear();
        state.options.push(Entry::new(Command::Text(
            "upgrade failed, check the logs".to_string(),
        )));
    }

    pub fn update_scanner_frame(&self, frame: DynamicImage) {
        let mut state = self.state.write().unwrap();
        if let Some(scanner) = &mut state.scanner {
            scanner.set_frame(frame);
        }
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

    pub fn lock_input(&self) {
        let mut state = self.state.write().unwrap();
        state.input.lock_input();
    }

    pub fn set_input_success(&self, msg: &str) -> bool {
        let mut state = self.state.write().unwrap();
        state.input.set_success(msg)
    }

    pub fn set_input_error(&self, msg: &str) -> bool {
        let mut state = self.state.write().unwrap();
        state.input.set_error(msg)
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
        if !self.is_visible() {
            return; // Do not render if popup is not active.
        }

        let mut state = self.state.write().unwrap();

        match state.mode {
            Mode::Menu => render_menu(area, buf, &mut state),
            Mode::Confirmation => render_confirm_and_sign(area, buf, &mut state),
            Mode::Transaction | Mode::Update => render_message(area, buf, &mut state),
            Mode::ChainSpecs => render_chain_specs_qrcode(area, buf, &mut state),
            Mode::Metadata => render_metadata_qrcode(area, buf, &mut state),
            Mode::Hidden => {}
        }
    }
}

fn render_menu(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let theme = CONFIG.theme();
    let options = state.get_options_filtered();

    let mode = state.mode.clone();
    let rows = options.iter().map(|f| {
        let command = f.get_command();
        to_row(command, mode.clone(), None)
    });

    // Split the area into top header to show all options, a small central box to show the input field
    // and a bottom footer to show the error message
    let details_len = (options.len() as u16 + 5).clamp(4, 10);
    let [top_area, details_area, input_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Max(details_len), // Top header
            Constraint::Length(5), // InputField as command mode (input (3) + invalid msg (2))
        ])
        .flex(Flex::End)
        .areas(area);

    let block = Block::new()
        .style(theme.block.pane_body)
        .padding(Padding::new(1, 1, 1, 0));

    let mut header_line = vec![];

    if state.title.is_some() {
        let title = Span::styled(
            state.title.as_deref().unwrap_or_default(),
            theme.paragraph.header(true),
        );
        header_line.push(title);
    }

    if state.label.is_some() {
        let label = Span::styled(
            format!(" ({})", state.label.as_deref().unwrap_or_default()),
            theme.paragraph.label(true),
        );
        header_line.push(label);
    }

    let header = Line::from(header_line).alignment(Alignment::Right);

    let top = Paragraph::new(header)
        .block(block)
        .wrap(Wrap { trim: false });

    Clear.render(top_area, buf);

    top.render(top_area, buf);

    let widths = [
        Constraint::Length(2),
        Constraint::Length(22),
        Constraint::Fill(2),
        Constraint::Length(2),
    ];

    let table_labels = vec!["", "command", "description", ""];

    let block = Block::new()
        .style(theme.block.pane_body)
        .padding(Padding::bottom(1));

    let table = Table::new(rows, widths)
        .block(block)
        .header(Row::new(table_labels).style(theme.table.header))
        .style(theme.table.base)
        .row_highlight_style(theme.table.row_highlight(true));

    Clear.render(details_area, buf);

    StatefulWidget::render(table, details_area, buf, &mut state.table_state);

    // Render input area
    let call = state.get_selected_call();
    state.input.as_command(call).render(input_area, buf);
}

fn render_confirm_and_sign(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let theme = CONFIG.theme();

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
        theme.paragraph.header(true),
    )])
    .alignment(Alignment::Right);

    let stash = Line::from(vec![
        Span::styled("stash ", theme.paragraph.label_inverse),
        Span::raw(stash_identity_entry.command()),
    ]);

    let method = Line::from(vec![
        Span::styled("method ", theme.paragraph.label_inverse),
        Span::raw(call_entry.to_method_truncated(32)),
    ]);

    let proxy = Line::from(vec![
        Span::styled("proxy account ", theme.paragraph.label_inverse),
        Span::raw(proxy_identity_entry.command()),
    ]);

    // Calculate spaces needed to show the `ctrl+shift+c copy on the right`
    let available_width = area.width.saturating_sub(4);
    let left_text = format!("call data {}", call_entry.to_hex_truncated(24));
    let right_len = "ctrl+shift+c copy".len() as u16; // 17
    let spaces = available_width
        .saturating_sub(left_text.len() as u16)
        .saturating_sub(right_len);

    let call_data = Line::from(vec![
        Span::styled("call data ", theme.paragraph.label_inverse),
        Span::raw(call_entry.to_hex_truncated(24)),
        Span::raw(" ".repeat(spaces as usize)),
        Span::styled("ctrl+shift+c", theme.paragraph.base),
        Span::raw(" "),
        Span::styled("copy", theme.paragraph.label_inverse),
    ]);

    let block = Block::new()
        .style(theme.block.pane_body)
        .padding(Padding::proportional(1));

    let details = Paragraph::new(vec![network, stash, method, proxy, call_data])
        .block(block)
        .wrap(Wrap { trim: false });

    // Note: The QR code entry is only available if QR code signing is enabled
    // If it's not available, we default to the standard sign mode
    match state.options.get(5) {
        Some(qr) => render_transaction_qrcode(qr.as_bytes(), details, area, buf, state),
        None => render_password_input(details, area, buf, state),
    }
}

fn render_transaction_qrcode(
    qr_bytes: Vec<u8>,
    details: Paragraph,
    area: Rect,
    buf: &mut Buffer,
    state: &mut PopupState,
) {
    let theme = CONFIG.theme();

    // Render qrcode
    let qrcode = QrCodeWidget::new(&qr_bytes);

    // Split the area into header to show transaction details and sign area (password / QR code)
    let [details_area, sign_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(7), // Details
            Constraint::Length(qrcode.height()),
        ])
        .flex(Flex::End)
        .areas(area);

    Clear.render(details_area, buf);
    Clear.render(sign_area, buf);

    details.render(details_area, buf);

    // Split the sign area into QR code and scanner camera area
    let [qrcode_area, scanner_area] = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(70), Constraint::Fill(1)])
        .areas(sign_area);

    let block = Block::default().style(theme.qrcode.base);

    qrcode
        .block(block)
        .style(theme.qrcode.base)
        .render(qrcode_area, buf);

    // Render qrcode scanner (camera)
    if let Some(ref mut scanner) = state.scanner {
        if let Some(ref mut frame) = scanner.frame_protocol {
            ScannerWidget::new(frame)
                .set_title("Scan QR code")
                .set_title_style(theme.qrcode.title)
                .set_style(theme.qrcode.scanner)
                .render(scanner_area, buf);
        }
    }
}

fn render_password_input(details: Paragraph, area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let [details_area, sign_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(7), // Details
            Constraint::Length(5),
        ])
        .flex(Flex::End)
        .areas(area);

    details.render(details_area, buf);

    state.input.as_password().render(sign_area, buf);
}

fn render_message(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let theme = CONFIG.theme();
    let horizontal = Layout::horizontal([Constraint::Max(56)]);
    let [area] = horizontal.areas(area);

    Clear.render(area, buf);

    let block = Block::new()
        .style(theme.block.main)
        .padding(Padding::proportional(1));

    let spinner_progress = state.spinner.status();

    let rows = state.options.iter().map(|f| {
        let command = f.get_command();
        to_row(command, state.mode.clone(), Some(&spinner_progress))
    });
    let widths = [Constraint::Fill(1), Constraint::Length(7)];
    let table = Table::new(rows, widths)
        .style(theme.table.base)
        .block(block);
    // .row_highlight_style(THEME.table.row_highlight(state.is_visible));

    StatefulWidget::render(table, area, buf, &mut state.table_state);
}

fn render_chain_specs_qrcode(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let theme = CONFIG.theme();

    let Some(legacy_name_entry) = state.options.first() else {
        return;
    };

    let Some(spec_version_entry) = state.options.get(1) else {
        return;
    };

    let Some(genesis_hash_entry) = state.options.get(2) else {
        return;
    };

    let Some(account_format_entry) = state.options.get(3) else {
        return;
    };

    let Some(unit_entry) = state.options.get(4) else {
        return;
    };

    let Some(qrcode_entry) = state.options.get(5) else {
        return;
    };

    let mut header_content = vec![];
    if state.title.is_some() {
        let title = Span::styled(
            state.title.as_deref().unwrap_or_default(),
            theme.paragraph.header(true),
        );
        header_content.push(title);
    }
    let header = Line::from(header_content).alignment(Alignment::Right);

    let runtime_version = Line::from(vec![
        Span::styled("runtime version ", theme.paragraph.label_inverse),
        Span::raw(format!(
            "{}/{}",
            legacy_name_entry.command(),
            spec_version_entry.command()
        )),
    ]);

    let genesis_hash = Line::from(vec![
        Span::styled("genesis hash ", theme.paragraph.label_inverse),
        Span::raw(genesis_hash_entry.to_hex()),
    ]);

    let account_format = Line::from(vec![
        Span::styled("address prefix ", theme.paragraph.label_inverse),
        Span::raw(account_format_entry.command()),
    ]);

    let unit = Line::from(vec![
        Span::styled("unit ", theme.paragraph.label_inverse),
        Span::raw(unit_entry.command()),
    ]);

    let block = Block::new()
        .style(theme.block.pane_header(true))
        .padding(Padding::proportional(1));

    let details = Paragraph::new(vec![
        header,
        runtime_version,
        genesis_hash,
        account_format,
        unit,
    ])
    .block(block)
    .wrap(Wrap { trim: false });

    // Render qrcode
    let qr_bytes = qrcode_entry.as_bytes();
    let qrcode = QrCodeWidget::new(&qr_bytes);

    // Split the area into header to chain details QR code
    let [details_area, qrcode_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(7), // Details
            Constraint::Length(qrcode.height()),
        ])
        .flex(Flex::End)
        .areas(area);

    Clear.render(details_area, buf);
    Clear.render(qrcode_area, buf);

    details.render(details_area, buf);

    let block = Block::default().style(theme.qrcode.base);

    qrcode
        .block(block)
        .style(theme.qrcode.base)
        .render(qrcode_area, buf);
}

fn render_metadata_qrcode(area: Rect, buf: &mut Buffer, state: &mut PopupState) {
    let theme = CONFIG.theme();
    let Some(legacy_name_entry) = state.options.first() else {
        return;
    };

    let Some(spec_version_entry) = state.options.get(1) else {
        return;
    };

    let Some(genesis_hash_entry) = state.options.get(2) else {
        return;
    };

    let Some(metadata_state) = state.metadata.as_ref() else {
        return;
    };

    let Some(qrcode) = metadata_state.frame() else {
        return;
    };

    let mut header_content = vec![];
    if state.title.is_some() {
        let title = Span::styled(
            state.title.as_deref().unwrap_or_default(),
            theme.paragraph.header(true),
        );
        header_content.push(title);
    }
    let header = Line::from(header_content).alignment(Alignment::Right);

    let runtime_version = Line::from(vec![
        Span::styled("runtime version ", theme.paragraph.label_inverse),
        Span::raw(format!(
            "{}/{}",
            legacy_name_entry.command(),
            spec_version_entry.command()
        )),
    ]);

    let genesis_hash = Line::from(vec![
        Span::styled("genesis_hash ", theme.paragraph.label_inverse),
        Span::raw(genesis_hash_entry.command()),
    ]);

    let block = Block::new()
        .style(theme.block.pane_header(true))
        .padding(Padding::proportional(1));

    let details = Paragraph::new(vec![header, runtime_version, genesis_hash])
        .block(block)
        .wrap(Wrap { trim: false });

    let qrcode = MetadataWidget::new(qrcode);

    // Split the area into header to chain details QR code
    let [details_area, qrcode_area] = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Max(7), // Details
            Constraint::Length(qrcode.height()),
        ])
        .flex(Flex::End)
        .areas(area);

    Clear.render(details_area, buf);
    Clear.render(qrcode_area, buf);

    details.render(details_area, buf);

    let block = Block::default().style(theme.qrcode.base);

    qrcode
        .block(block)
        .style(theme.qrcode.base)
        .render(qrcode_area, buf);
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
            Mode::Transaction | Mode::Update => {
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
        _ => Row::new(vec!["".to_string()]),
    }
}

fn _calculate_text_wrapped_lines(text: &str, area_width: u16) -> u16 {
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
