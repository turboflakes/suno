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
use suno_actions::{ChainSpecsContext, ConfirmationContext, MetadataContext, ThreadAction};
use suno_config::{SupportedRuntime, CONFIG};
use suno_primitives::{
    call::Call,
    entry::{Command, Entry, ToDescription, ToMethod},
    session::{Keys, Proof},
    staking::Payee,
    Chain, Validator,
};
use suno_qrcode::{MetadataState, MetadataWidget, QrCodeWidget, ScannerWidget};
use tokio::sync::mpsc::UnboundedSender;
use unicode_width::UnicodeWidthStr;

type ActiveEra = u32;

#[derive(Clone, PartialEq, Eq)]
struct ValidatorContext {
    era: ActiveEra,
    validator: Validator,
}

struct ChainContext {
    runtime: SupportedRuntime,
}

#[derive(Clone, PartialEq, Eq)]
struct MessageContext {
    msg: String,
}

impl MessageContext {
    fn new(msg: impl Into<String>) -> Self {
        Self { msg: msg.into() }
    }
}

#[derive(Default)]
enum Context {
    #[default]
    None,
    ValidatorMenu(Box<ValidatorContext>),
    ChainMenu(Box<ChainContext>),
    Confirmation(Box<ConfirmationContext>),
    Message(MessageContext),
    ChainSpecs(Box<ChainSpecsContext>),
    Metadata(Box<MetadataContext>),
}

impl Context {
    fn mode(&self) -> Mode {
        match self {
            Context::None => Mode::Hidden,
            Context::ValidatorMenu(_) => Mode::Menu,
            Context::ChainMenu(_) => Mode::Menu,
            Context::Confirmation(_) => Mode::Confirmation,
            Context::Message(_) => Mode::Message,
            Context::ChainSpecs(_) => Mode::ChainSpecs,
            Context::Metadata(_) => Mode::Metadata,
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Mode {
    #[default]
    Hidden,
    Menu,
    Confirmation,
    Message,
    ChainSpecs,
    Metadata,
}

pub struct ScannerSession {
    _ctrl: UnboundedSender<ThreadAction>,
    frame_protocol: Option<StatefulProtocol>,
}

impl ScannerSession {
    pub fn new(ctrl: UnboundedSender<ThreadAction>) -> Self {
        Self {
            _ctrl: ctrl,
            frame_protocol: None,
        }
    }
}

pub struct Popup {
    context: Context,
    options: Vec<Entry<Call>>,
    table_state: TableState,
    input: InputFieldWidget,
    spinner: Spinner,
    scanner: Option<ScannerSession>,
    picker: Picker,
    masked: bool,
    metadata: Option<MetadataState>,
}

impl Default for Popup {
    fn default() -> Self {
        let picker = Picker::from_query_stdio().unwrap_or_else(|_| Picker::halfblocks());
        Self {
            context: Context::default(),
            options: Vec::new(),
            table_state: TableState::default(),
            input: InputFieldWidget::new(),
            spinner: Spinner::default(),
            scanner: None,
            picker,
            masked: true,
            metadata: None,
        }
    }
}

impl Popup {
    fn on_init(&mut self, context: Context) {
        self.options.clear();
        self.metadata = None;
        self.scanner = None;
        self.table_state.select(None);

        match &context {
            Context::ValidatorMenu(ctx) => self.init_validator_menu(ctx),
            Context::ChainMenu(ctx) => self.init_chain_menu(ctx),
            Context::Confirmation(ctx) => {
                if !ctx.runtime.is_qrcode_enabled() {
                    self.input.reset_as_password();
                }
            }
            Context::Message(_) => {
                self.spinner.increment();
            }
            Context::ChainSpecs(_) => {}
            Context::Metadata(ctx) => {
                self.metadata = Some(MetadataState::new(&ctx.qr_bytes));
            }
            Context::None => {}
        }

        self.context = context;
    }

    fn init_validator_menu(&mut self, ctx: &ValidatorContext) {
        if !ctx.validator.is_proxy_valid() && !ctx.validator.is_commands_available() {
            return;
        }

        let runtime = ctx.validator.runtime().asset_hub_runtime();
        let unit = runtime.token_symbol();
        let decimals = runtime.token_decimals();
        let metadata = InputFieldMetadata::new()
            .with_unit(unit)
            .with_decimals(decimals)
            .with_custom_commands(ctx.validator.commands.clone());
        self.input.reset_as_command(Some(metadata));

        ctx.validator.proxies.iter().for_each(|p| {
            let bond = Call::Bond {
                amount: 0,
                payee: Payee::default(),
                max: Some(ctx.validator.free_balance_extended(4)),
            };
            if p.proxy().can_call(&bond) && ctx.validator.is_unknown() {
                self.options.push(Entry::new(Command::Instruction {
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
                self.options.push(Entry::new(Command::Instruction {
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
                self.options.push(Entry::new(Command::Instruction {
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
                self.options.push(Entry::new(Command::Instruction {
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
                self.options.push(Entry::new(Command::Instruction {
                    call: withdraw,
                    bytes: None,
                }));
            }

            let set_payee = Call::SetPayee {
                payee: Payee::default(),
            };
            if p.proxy().can_call(&set_payee) && ctx.validator.is_active_or_waiting() {
                self.options.push(Entry::new(Command::Instruction {
                    call: set_payee,
                    bytes: None,
                }));
            }

            let validate = Call::Validate {
                commission: Perbill::from_percent(0),
                blocked: false,
            };
            if p.proxy().can_call(&validate) {
                self.options.push(Entry::new(Command::Instruction {
                    call: validate,
                    bytes: None,
                }));
            }

            let chill = Call::Chill;
            if p.proxy().can_call(&chill) && ctx.validator.is_active_or_waiting() {
                self.options.push(Entry::new(Command::Instruction {
                    call: chill,
                    bytes: None,
                }));
            }

            let set_keys = Call::SetKeys {
                keys: Keys::default(),
                proof: Proof::default(),
            };
            if p.proxy().can_call(&set_keys) && ctx.validator.is_active_or_waiting() {
                self.options.push(Entry::new(Command::Instruction {
                    call: set_keys,
                    bytes: None,
                }));
            }

            let purge_keys = Call::PurgeKeys;
            if p.proxy().can_call(&purge_keys)
                && ctx.validator.is_active_or_waiting()
                && ctx.validator.has_keys()
            {
                self.options.push(Entry::new(Command::Instruction {
                    call: purge_keys,
                    bytes: None,
                }));
            }
        });

        ctx.validator.commands.iter().for_each(|c| {
            self.options.push(Entry::new(Command::Instruction {
                call: Call::Custom(c.clone()),
                bytes: None,
            }));
        });

        if !self.options.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    fn init_chain_menu(&mut self, ctx: &ChainContext) {
        self.input.reset_as_command(None);

        self.options.push(Entry::new(Command::Instruction {
            call: Call::ChainSpecs {
                chain_name: ctx.runtime.to_string(),
            },
            bytes: None,
        }));

        self.options.push(Entry::new(Command::Instruction {
            call: Call::Metadata {
                chain_name: ctx.runtime.to_string(),
            },
            bytes: None,
        }));

        if !self.options.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn show_validator_commands(&mut self, validator: &Validator, active_era: ActiveEra) {
        let ctx = ValidatorContext {
            era: active_era,
            validator: validator.clone(),
        };
        self.on_init(Context::ValidatorMenu(Box::new(ctx)));
    }

    pub fn show_chain_commands(&mut self, chain: &Chain) {
        let ctx = ChainContext {
            runtime: chain.runtime(),
        };
        self.on_init(Context::ChainMenu(Box::new(ctx)));
    }

    pub fn show_confirm_and_sign(&mut self, ctx: &ConfirmationContext) {
        self.on_init(Context::Confirmation(Box::new(ctx.clone())));
    }

    pub fn show_chain_specs_qrcode(&mut self, ctx: &ChainSpecsContext) {
        self.on_init(Context::ChainSpecs(Box::new(ctx.clone())));
    }

    pub fn show_metadata_qrcode(&mut self, ctx: &MetadataContext) {
        self.on_init(Context::Metadata(Box::new(ctx.clone())));
    }

    pub fn show_message(&mut self, msg: impl Into<String>) {
        self.on_init(Context::Message(MessageContext::new(msg)));
    }

    pub fn advance_metadata_frame(&mut self) {
        if let Some(metadata) = self.metadata.as_mut() {
            metadata.advance_frame();
        }
    }

    pub fn is_hidden(&self) -> bool {
        matches!(self.context, Context::None)
    }

    pub fn is_visible(&self) -> bool {
        !self.is_hidden()
    }

    pub fn move_down(&mut self) -> Option<Entry<Call>> {
        let options = self.get_options_filtered();
        if options.is_empty() {
            self.table_state.select(None);
            return None;
        }

        let selected = self
            .table_state
            .selected()
            .unwrap_or(0)
            .min(options.len() - 1);
        let next = if selected == options.len() - 1 {
            0
        } else {
            selected + 1
        };
        self.table_state.select(Some(next));
        options.get(next).cloned()
    }

    pub fn move_up(&mut self) -> Option<Entry<Call>> {
        let options = self.get_options_filtered();
        if options.is_empty() {
            self.table_state.select(None);
            return None;
        }

        let selected = self
            .table_state
            .selected()
            .unwrap_or(0)
            .min(options.len() - 1);
        let next = if selected == 0 {
            options.len() - 1
        } else {
            selected - 1
        };
        self.table_state.select(Some(next));
        options.get(next).cloned()
    }

    pub fn close(&mut self) {
        self.context = Context::None;
        self.options.clear();
        self.scanner = None;
        self.metadata = None;
        self.table_state.select(None);
        self.input.clear_focus();
    }

    pub fn start_scanner(&mut self, ctrl: UnboundedSender<ThreadAction>) {
        self.scanner = Some(ScannerSession::new(ctrl));
    }

    pub fn get_selected(&self) -> Option<Entry<Call>> {
        let options = self.get_options_filtered();
        if options.is_empty() {
            return None;
        }
        self.table_state.selected().and_then(|i| {
            let i = i.min(options.len() - 1);
            options.get(i).cloned()
        })
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

    pub fn get_selected_call(&self) -> Option<Call> {
        self.get_selected()
            .and_then(|selected| match selected.get_command() {
                Command::Instruction { call, .. } => Some(call),
                _ => None,
            })
    }

    pub fn get_call_data_bytes(&self) -> Option<Vec<u8>> {
        if let Context::Confirmation(ctx) = &self.context {
            Some(ctx.call_data_bytes.clone())
        } else {
            None
        }
    }

    pub fn get_input_parsed_call(&self) -> Option<Call> {
        self.input.get_parsed_call()
    }

    pub fn get_mode(&self) -> Mode {
        self.context.mode()
    }

    pub fn is_confirmation_mode(&self) -> bool {
        matches!(self.context, Context::Confirmation(_))
    }

    pub fn is_menu_mode(&self) -> bool {
        matches!(
            self.context,
            Context::ValidatorMenu(_) | Context::ChainMenu(_)
        )
    }

    pub fn can_close(&self) -> bool {
        matches!(
            self.context,
            Context::ValidatorMenu(_)
                | Context::ChainMenu(_)
                | Context::ChainSpecs(_)
                | Context::Metadata(_)
                | Context::Confirmation(_)
        )
    }

    pub fn is_masked(&self) -> bool {
        self.masked
    }

    pub fn toggle_mask(&mut self) {
        self.masked = !self.is_masked();
    }

    pub fn update_message(&mut self, msg: impl Into<String>) {
        self.spinner.increment();
        self.context = Context::Message(MessageContext::new(msg));
    }

    pub fn show_upgrade_complete(&mut self, msg: impl Into<String>) {
        self.spinner.complete();
        self.context = Context::Message(MessageContext::new(msg));
    }

    pub fn show_upgrade_error(&mut self) {
        self.spinner.error();
        self.context = Context::Message(MessageContext::new("upgrade failed, check the logs"));
    }

    pub fn update_scanner_frame(&mut self, frame: DynamicImage) {
        let frame_protocol = self.picker.new_resize_protocol(frame);
        if let Some(scanner) = self.scanner.as_mut() {
            scanner.frame_protocol = Some(frame_protocol);
        }
    }

    pub fn set_input_focus(&mut self) -> bool {
        self.input.set_focus()
    }

    pub fn clear_input_focus(&mut self) {
        self.input.clear_focus();
    }

    pub fn lock_input(&mut self) {
        self.input.lock_input();
    }

    pub fn set_input_success(&mut self, msg: &str) -> bool {
        self.input.set_success(msg)
    }

    pub fn set_input_error(&mut self, msg: &str) -> bool {
        self.input.set_error(msg)
    }

    pub fn invalidate_input(&mut self, msg: &str) -> bool {
        self.input.invalidate(msg)
    }

    pub fn insert_input_char(&mut self, new_char: char) {
        self.input.insert_char(new_char);
    }

    pub fn delete_input_char(&mut self) {
        self.input.delete_char();

        if self.is_menu_mode() {
            let options = self.get_options_filtered();
            if options.is_empty() {
                self.table_state.select(None);
                return;
            }
            self.table_state.select(Some(0));
        }
    }

    pub fn insert_input_paste_data(&mut self, data: String) {
        self.input.paste_data(data);
    }

    pub fn move_cursor_left(&mut self) {
        self.input.move_cursor_left();
    }

    pub fn move_cursor_right(&mut self) {
        self.input.move_cursor_right();
    }

    pub fn set_input_autocomplete(&mut self) {
        if let Some(call) = self.get_selected_call() {
            self.input.set_value(call.to_string());
        }
    }

    pub fn execute_with_password<F, R, E>(&self, f: F) -> Result<R, E>
    where
        F: FnOnce(&str) -> Result<R, E>,
    {
        self.input.execute_with_password(f)
    }

    pub fn get_input_cursor_position(&self) -> Option<Position> {
        self.input.get_cursor_position()
    }
}

impl Widget for &mut Popup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        match self.get_mode() {
            Mode::Menu => self.render_menu(area, buf),
            Mode::Confirmation => self.render_confirm_and_sign(area, buf),
            Mode::Message => self.render_message(area, buf),
            Mode::ChainSpecs => self.render_chain_specs_qrcode(area, buf),
            Mode::Metadata => self.render_metadata_qrcode(area, buf),
            Mode::Hidden => {}
        }
    }
}

impl Popup {
    fn render_menu(&mut self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();
        let options = self.get_options_filtered();
        let rows = options
            .iter()
            .map(|entry| to_row(entry.get_command(), self.get_mode()));

        let (title, label) = match &self.context {
            Context::ValidatorMenu(ctx) => {
                let label = if !ctx.validator.commands.is_empty() {
                    format!("Commands ({})", ctx.validator.host(self.masked))
                } else {
                    "Commands".to_string()
                };
                (Some(ctx.validator.display_identity()), Some(label))
            }
            Context::ChainMenu(ctx) => (
                Some(ctx.runtime.as_str_long().to_uppercase()),
                Some("Commands".to_string()),
            ),
            _ => (None, None),
        };

        let details_len = (options.len() as u16 + 5).clamp(4, 10);
        let [top_area, details_area, input_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(2),
                Constraint::Max(details_len),
                Constraint::Length(5),
            ])
            .flex(Flex::End)
            .areas(area);

        let block = Block::new()
            .style(theme.block.popup_header)
            .padding(Padding::new(1, 1, 1, 0));

        let mut header_line = vec![];

        if let Some(label) = label {
            header_line.push(Span::styled(
                format!("{} ", label),
                theme.paragraph.label(true),
            ));
        }

        if let Some(title) = title {
            header_line.push(Span::styled(title, theme.paragraph.header(true)));
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

        let block = Block::new()
            .style(theme.block.popup_header)
            .padding(Padding::bottom(1));

        let table = Table::new(rows, widths)
            .block(block)
            .header(Row::new(["", "command", "description", ""]).style(theme.table.header))
            .style(theme.table.base)
            .row_highlight_style(theme.table.row_highlight(true));

        Clear.render(details_area, buf);
        StatefulWidget::render(table, details_area, buf, &mut self.table_state);

        let call = self.get_selected_call();
        self.input.as_command(call).render(input_area, buf);
    }

    fn render_confirm_and_sign(&mut self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();

        let Context::Confirmation(ctx) = &self.context else {
            return;
        };

        let title = if ctx.runtime.is_qrcode_enabled() {
            "SCAN TO AUTHORIZE TRANSACTION"
        } else {
            "AUTHORIZE TRANSACTION"
        };
        let runtime_version_value = format!("{}/{}", ctx.runtime.legacy_name(), ctx.spec_version);
        let stash = ctx.stash_identity.clone();
        let proxy = ctx.proxy_identity.clone();
        let method = truncate_method(&ctx.call, 32);
        let call_data = truncate_hex(&ctx.call_data_bytes, 24);
        let qr_bytes = if ctx.runtime.is_qrcode_enabled() {
            Some(ctx.qr_bytes.clone())
        } else {
            None
        };

        let header = Line::from(vec![Span::styled(title, theme.paragraph.header(true))])
            .alignment(Alignment::Right);
        let runtime_version = Line::from(vec![
            Span::styled("runtime version ", theme.paragraph.label_inverse),
            Span::raw(runtime_version_value),
        ]);
        let stash = Line::from(vec![
            Span::styled("stash ", theme.paragraph.label_inverse),
            Span::raw(stash),
        ]);
        let method = Line::from(vec![
            Span::styled("method ", theme.paragraph.label_inverse),
            Span::raw(method),
        ]);
        let proxy = Line::from(vec![
            Span::styled("proxy account ", theme.paragraph.label_inverse),
            Span::raw(proxy),
        ]);

        let available_width = usize::from(area.width.saturating_sub(4));
        let left_text = format!("call data {call_data}");
        let right_text = "ctrl+shift+c copy";
        let spaces = available_width
            .saturating_sub(left_text.width())
            .saturating_sub(right_text.width());

        let call_data = Line::from(vec![
            Span::styled("call data ", theme.paragraph.label_inverse),
            Span::raw(call_data),
            Span::raw(" ".repeat(spaces)),
            Span::styled("ctrl+shift+c", theme.paragraph.base),
            Span::raw(" "),
            Span::styled("copy", theme.paragraph.label_inverse),
        ]);

        let details = Paragraph::new(vec![
            header,
            runtime_version,
            stash,
            method,
            proxy,
            call_data,
        ])
        .block(
            Block::new()
                .style(theme.block.popup_header)
                .padding(Padding::proportional(1)),
        )
        .wrap(Wrap { trim: false });

        match qr_bytes {
            Some(qr_bytes) => self.render_transaction_qrcode(&qr_bytes, details, area, buf),
            None => self.render_password_input(details, area, buf),
        }
    }

    fn render_transaction_qrcode(
        &mut self,
        qr_bytes: &[u8],
        details: Paragraph<'_>,
        area: Rect,
        buf: &mut Buffer,
    ) {
        let theme = CONFIG.theme();

        let qrcode = QrCodeWidget::new(qr_bytes);

        let [details_area, sign_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(7), Constraint::Length(qrcode.height())])
            .flex(Flex::End)
            .areas(area);

        Clear.render(details_area, buf);
        Clear.render(sign_area, buf);
        details.render(details_area, buf);

        let [qrcode_area, scanner_area] = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Fill(1)])
            .areas(sign_area);

        qrcode
            .block(Block::default().style(theme.qrcode.base))
            .style(theme.qrcode.base)
            .render(qrcode_area, buf);

        if let Some(scanner) = self.scanner.as_mut() {
            if let Some(frame) = scanner.frame_protocol.as_mut() {
                ScannerWidget::new(frame)
                    .set_title("Scan QR code")
                    .set_title_style(theme.qrcode.title)
                    .set_style(theme.qrcode.scanner)
                    .render(scanner_area, buf);
            }
        }
    }

    fn render_password_input(&self, details: Paragraph<'_>, area: Rect, buf: &mut Buffer) {
        let [details_area, sign_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(7), Constraint::Length(5)])
            .flex(Flex::End)
            .areas(area);

        details.render(details_area, buf);
        self.input.as_password().render(sign_area, buf);
    }

    fn render_message(&mut self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();

        let Context::Message(ctx) = &self.context else {
            return;
        };

        let [area] = Layout::horizontal([Constraint::Max(56)]).areas(area);

        Clear.render(area, buf);

        let row = Row::new([
            Cell::from(Line::from(ctx.msg.clone())),
            Cell::from(Line::from(self.spinner.status()).alignment(Alignment::Right)),
        ]);

        let table = Table::new([row], [Constraint::Fill(1), Constraint::Length(7)])
            .style(theme.table.base)
            .block(
                Block::new()
                    .style(theme.block.main)
                    .padding(Padding::proportional(1)),
            );

        StatefulWidget::render(table, area, buf, &mut self.table_state);
    }

    fn render_chain_specs_qrcode(&self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();

        let Context::ChainSpecs(ctx) = &self.context else {
            return;
        };

        let runtime_version_value = format!("{}/{}", ctx.runtime.legacy_name(), ctx.spec_version);
        let genesis_hash_value = format!("0x{}", hex::encode(ctx.runtime.chain_genesis_hash().0));
        let account_format_value = ctx.runtime.account_format().to_string();
        let unit_value = ctx.runtime.token_symbol().to_string();
        let qr_bytes = ctx.qr_bytes.clone();

        let header = Line::from(vec![Span::styled(
            "SCAN TO ADD CHAIN SPECS",
            theme.paragraph.header(true),
        )])
        .alignment(Alignment::Right);
        let runtime_version = Line::from(vec![
            Span::styled("runtime version ", theme.paragraph.label_inverse),
            Span::raw(runtime_version_value),
        ]);
        let genesis_hash = Line::from(vec![
            Span::styled("genesis hash ", theme.paragraph.label_inverse),
            Span::raw(genesis_hash_value),
        ]);
        let account_format = Line::from(vec![
            Span::styled("address prefix ", theme.paragraph.label_inverse),
            Span::raw(account_format_value),
        ]);
        let unit = Line::from(vec![
            Span::styled("unit ", theme.paragraph.label_inverse),
            Span::raw(unit_value),
        ]);

        let details = Paragraph::new(vec![
            header,
            runtime_version,
            genesis_hash,
            account_format,
            unit,
        ])
        .block(
            Block::new()
                .style(theme.block.popup_header)
                .padding(Padding::proportional(1)),
        )
        .wrap(Wrap { trim: false });

        let qrcode = QrCodeWidget::new(&qr_bytes);
        let [details_area, qrcode_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(7), Constraint::Length(qrcode.height())])
            .flex(Flex::End)
            .areas(area);

        Clear.render(details_area, buf);
        Clear.render(qrcode_area, buf);
        details.render(details_area, buf);

        qrcode
            .block(Block::default().style(theme.qrcode.base))
            .style(theme.qrcode.base)
            .render(qrcode_area, buf);
    }

    fn render_metadata_qrcode(&mut self, area: Rect, buf: &mut Buffer) {
        let theme = CONFIG.theme();

        let Context::Metadata(ctx) = &self.context else {
            return;
        };

        let runtime_version_value = format!("{}/{}", ctx.runtime.legacy_name(), ctx.spec_version);
        let genesis_hash_value = format!("0x{}", hex::encode(ctx.runtime.chain_genesis_hash().0));

        let Some(metadata_state) = self.metadata.as_ref() else {
            return;
        };
        let Some(qrcode) = metadata_state.frame() else {
            return;
        };

        let header = Line::from(vec![Span::styled(
            "SCAN TO UPDATE METADATA",
            theme.paragraph.header(true),
        )])
        .alignment(Alignment::Right);
        let runtime_version = Line::from(vec![
            Span::styled("runtime version ", theme.paragraph.label_inverse),
            Span::raw(runtime_version_value),
        ]);
        let genesis_hash = Line::from(vec![
            Span::styled("genesis_hash ", theme.paragraph.label_inverse),
            Span::raw(genesis_hash_value),
        ]);

        let details = Paragraph::new(vec![header, runtime_version, genesis_hash])
            .block(
                Block::new()
                    .style(theme.block.popup_header)
                    .padding(Padding::proportional(1)),
            )
            .wrap(Wrap { trim: false });

        let qrcode = MetadataWidget::new(qrcode);
        let [details_area, qrcode_area] = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(7), Constraint::Length(qrcode.height())])
            .flex(Flex::End)
            .areas(area);

        Clear.render(details_area, buf);
        Clear.render(qrcode_area, buf);
        details.render(details_area, buf);

        qrcode
            .block(Block::default().style(theme.qrcode.base))
            .style(theme.qrcode.base)
            .render(qrcode_area, buf);
    }
}

pub fn to_row(command: Command<Call>, mode: Mode) -> Row<'static> {
    match command {
        Command::Instruction { call, .. } => {
            let mut cols = Vec::new();

            if mode == Mode::Menu {
                cols.push("".to_string());
                cols.push(format!("/{call}"));
                cols.push(call.description());
                cols.push("".to_string());
            }

            Row::new(cols)
        }
        Command::Text(text) => Row::new(vec![text.to_string()]),
        _ => Row::new(vec!["".to_string()]),
    }
}

fn truncate_method(call: &Call, max_length: usize) -> String {
    let method = call.to_method();
    if method.len() > max_length {
        format!("{}..", method.chars().take(max_length).collect::<String>())
    } else {
        method
    }
}

fn truncate_hex(bytes: &[u8], max_length: usize) -> String {
    let encoded = hex::encode(bytes);
    if encoded.len() > max_length {
        format!(
            "0x{}..",
            encoded.chars().take(max_length).collect::<String>()
        )
    } else {
        format!("0x{encoded}")
    }
}
