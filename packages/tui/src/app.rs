use crate::bridge::{custom, sync, RuntimeCaller, RuntimeFetcher};
use crate::section::Section;
use crate::widgets::{
    chains::ChainsList,
    collators::CollatorsList,
    help::HelpState,
    input_field::InputFieldWidget,
    list_focus::ListFocus,
    logs::LogsState,
    nominators::NominatorsList,
    popup::{Mode as PopupMode, Popup},
    proxies::ProxiesList,
    validators::ValidatorsList,
    window::Window,
};
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use arboard::Clipboard;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::str::{from_utf8, FromStr};
use std::{collections::HashSet, io, thread, time::Duration};
use subxt::{utils::AccountId32, OnlineClient, OnlineClientAtBlock};
use suno_actions::{
    Action, ChainAction, ChainSpecsContext, CollatorAction, ConfirmationContext, FilterAction,
    InputAction, MetadataContext, NavigationAction, NominatorAction, PopupAction, ScannerAction,
    SystemAction, ThreadAction, TxAction, UpdateAction, ValidatorAction,
};
use suno_config::{
    remove_proxy_account, save_active_theme, save_feature, save_pinned_collators,
    save_pinned_nominators, save_pinned_validators, save_proxy_account, CommandKind, CustomCalls,
    CustomCommand, CustomConfig, Feature, NodeAccess, SupportedRuntime, CONFIG,
};
use suno_error::{Error, ResultExt};
use suno_primitives::{
    call::Call, display::to_compact_string, entry::ToMethod, network::ConnectionState,
    proxy::ProxyState, AccountDisplay, AccountKey, BlockHash, Chain, Collator, NodeHost, Validator,
};
use suno_qrcode::{
    build::{
        build_chain_specs_qrcode_signed, build_chain_specs_qrcode_unsigned, build_metadata_payload,
        build_metadata_qrcode_signed, build_metadata_qrcode_unsigned, build_transaction_qrcode,
        extract_address_from_qr,
    },
    scanner::Scanner,
    NetworkSpecsToSend,
};
use suno_theme::Theme;
use suno_tracing::LogEntry;
use suno_update::update;
use tokio::sync::mpsc;
use tracing::{error, info};
use zeroize::Zeroizing;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Error>;

/// The tick rate at 100 ms represents 10 FPS
const TICK_RATE: u64 = 100;

/// Application active focus.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Focus {
    #[default]
    Main, // Arrows move the sections and tabs
    Input,   // Arrows move the cursor in the input/password fields
    Filter,  // Typing filters the list of the window displayed
    Scanner, // Esc/Tab change mode state
    Popup,   // Esc/Tab change mode state
    None,    // Nothing is selected, after `esc` clears the selections
}

/// Thread control messages, useful for controlling threads from the app, eg. stopping a scanner thread.
pub enum ThreadControl {
    Stop,
}

/// Application.
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Application active focus, useful to determine how to handle keyboard events.
    pub focus: Focus,
    /// The current selected window.
    pub window: Window,
    /// The current selected section.
    pub section: Section,
    /// The section whose rows are shown and selected on the main window, while the main
    /// window itself is the selected section.
    main_section: Section,
    /// Holds the API clients for each supported runtime.
    pub chains: ChainsList,
    pub proxies: ProxiesList,
    /// Holds the nominators listed in the config.
    pub nominators: NominatorsList,
    /// Holds the validators list for the selected relay-chain.
    pub validators: ValidatorsList,
    /// Holds the collators list for the selected relay-chain.
    pub collators: CollatorsList,
    /// Manages popup state and rendering.
    pub popup: Popup,
    /// Logs state.
    pub logs: LogsState,
    /// Help state.
    pub help: HelpState,
    /// Is any sensitive data masked?
    pub masked: bool,
    /// The currently active theme, picked from the `ctrl+t` menu.
    pub theme: Theme,
    /// Index of the active theme within `CONFIG.theme_catalog()`.
    theme_index: usize,
    /// Theme index to restore if the theme menu is cancelled.
    theme_index_before_menu: usize,
    /// New version available
    pub new_version: Option<String>,
    /// The sender to send actions to update the state to the app.
    pub tx: mpsc::UnboundedSender<Action>,
    /// The receiver to handle actions sent from tx.
    pub rx: mpsc::UnboundedReceiver<Action>,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new(rx_logs: mpsc::UnboundedReceiver<LogEntry>) -> Self {
        // Define the channel to send actions to update the app state.
        let (tx, rx) = mpsc::unbounded_channel::<Action>();

        let theme_index = CONFIG.theme_active_index();
        let theme = CONFIG
            .theme_catalog()
            .get(theme_index)
            .map(|(_, theme)| *theme)
            .unwrap_or_default();

        let mut chains = ChainsList::default();
        chains.set_theme(theme);
        let mut proxies = ProxiesList::default();
        proxies.set_theme(theme);
        let mut nominators = NominatorsList::default();
        nominators.set_theme(theme);
        let mut popup = Popup::default();
        popup.set_theme(theme);
        let mut logs = LogsState::new(rx_logs);
        logs.set_theme(theme);
        let mut help = HelpState::default();
        help.set_theme(theme);

        Self {
            running: true,
            focus: Focus::default(),
            window: Window::first(CONFIG.features()),
            // Starts on the tabs of the main window, with the first window enabled displayed
            section: Section::None,
            main_section: Section::default(),
            chains,
            proxies,
            nominators,
            validators: ValidatorsList::default(),
            collators: CollatorsList::default(),
            popup,
            logs,
            help,
            masked: true,
            theme,
            theme_index,
            theme_index_before_menu: theme_index,
            new_version: None,
            tx,
            rx,
        }
    }

    async fn init(&mut self) {
        self.update_visible_lists();
        self.update_active_section();
        self.validators.on_init().await;
        self.collators.on_init();
        self.nominators.on_init();
        self.refresh_proxies();
        self.chains.on_init(self.tx.clone()).await;
        self.check_for_update();
    }

    fn is_masked(&self) -> bool {
        self.masked
    }

    fn check_for_update(&mut self) {
        let features = CONFIG.features();
        if !features.check_update_enabled() {
            return;
        }

        let tx = self.tx.clone();
        tokio::spawn(async move {
            let res = suno_update::check_for_update().await;
            match res {
                Ok(version) => {
                    let _ = tx.send(Action::Update(UpdateAction::NewVersionAvailable(version)));
                }
                Err(suno_update::Error::AlreadyUpToDate) => (),
                Err(e) => error!("{:?}", e),
            }
        });
    }

    pub async fn run(&mut self) -> AppResult<()> {
        // Initialize async widgets
        self.init().await;
        // Initialize the terminal user interface.
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)?;
        let events = EventHandler::new(TICK_RATE);
        let mut tui = Tui::new(terminal, events);
        tui.init()?;

        // Start the main loop.
        while self.running {
            // Drain tracing events
            self.logs.update();
            // Render the user interface.
            tui.draw(self)?;
            // Handle events.
            let event = tui.events.next().await?;
            self.handle_events(event)?;
            // Handle actions.
            self.handle_actions();
        }

        // Exit the user interface.
        tui.exit()?;
        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> AppResult<()> {
        let action = match event {
            Event::Tick => Action::System(SystemAction::Tick),
            Event::Key(key_event) => handle_key_events(key_event, self.focus.clone()),
            Event::Paste(data) if self.focus == Focus::Filter => {
                Action::Filter(FilterAction::Paste(data))
            }
            Event::Paste(data) => Action::Input(InputAction::Paste(data)),
            Event::Mouse(_) => Action::System(SystemAction::Noop),
            Event::Resize(_, _) => Action::System(SystemAction::Noop),
            // _ => Action::System(SystemAction::Noop),
        };
        self.tx.send(action.clone()).boxed()?;
        Ok(())
    }

    fn handle_actions(&mut self) {
        while let Ok(action) = self.rx.try_recv() {
            // Apply actionable messages to the application.
            match action {
                Action::System(act) => self.handle_system_actions(act),
                Action::Navigation(act) => self.handle_navigation_actions(act),
                Action::Popup(act) => self.handle_popup_actions(act),
                Action::Input(act) => self.handle_input_actions(act),
                Action::Filter(act) => self.handle_filter_actions(act),
                Action::Chain(act) => self.handle_chain_actions(act),
                Action::Validator(act) => self.handle_validator_actions(act),
                Action::Collator(act) => self.handle_collator_actions(act),
                Action::Nominator(act) => self.handle_nominator_actions(act),
                Action::Transaction(act) => self.handle_transaction_actions(act),
                Action::Scanner(act) => self.handle_scanner_actions(act),
                Action::Update(act) => self.handle_update_actions(act),
            }
        }
    }

    fn handle_system_actions(&mut self, action: SystemAction) {
        match action {
            SystemAction::Quit => self.quit(),
            SystemAction::Update => {
                let _ = self.tx.send(Action::Update(UpdateAction::Start));
            }
            SystemAction::Tick => self.tick(),
            SystemAction::Noop => self.noop(),
            SystemAction::Error(err) => error!("{err}"),
        }
    }

    fn handle_navigation_actions(&mut self, action: NavigationAction) {
        match action {
            NavigationAction::SectionUp => self.section_up(),
            NavigationAction::SectionDown => self.section_down(),
            NavigationAction::PrevTab => self.prev_tab(),
            NavigationAction::NextTab => self.next_tab(),
            NavigationAction::ExtendUp => self.extend_up(),
            NavigationAction::ExtendDown => self.extend_down(),
            NavigationAction::MoveUp => self.move_up(),
            NavigationAction::MoveDown => self.move_down(),
            NavigationAction::NextWindow => self.next_window(),
            NavigationAction::PrevWindow => self.prev_window(),
            NavigationAction::Reset => self.reset_selection(),
            NavigationAction::Copy => self.copy_to_clipboard(),
            NavigationAction::ToggleMask => self.toggle_mask(),
            NavigationAction::ToggleView => self.toggle_view(),
            NavigationAction::TogglePin => self.toggle_pin(),
        }
    }

    fn handle_popup_actions(&mut self, action: PopupAction) {
        match action {
            PopupAction::Open => self.open_popup(),
            PopupAction::OpenThemeMenu => self.open_theme_menu(),
            PopupAction::OpenFeatures => self.open_features(),
            PopupAction::ToggleFeature => self.toggle_feature(),
            PopupAction::ResetFeature => self.reset_feature(),
            PopupAction::Confirm => self.confirm_popup(),
            PopupAction::ShowConfirmAndSign(ctx) => {
                self.open_confirm_and_sign_popup(&ctx);
            }
            PopupAction::ShowChainSpecsQrcode(ctx) => {
                self.popup.show_chain_specs_qrcode(&ctx);
            }
            PopupAction::ShowMetadataQrcode(ctx) => {
                self.popup.show_metadata_qrcode(&ctx);
            }
            PopupAction::Close => self.close_popup(),
            PopupAction::Cancel => self.cancel(),
        }
    }

    fn handle_input_actions(&mut self, action: InputAction) {
        if !self.popup.is_visible() {
            return;
        }
        match action {
            InputAction::Editing(start_char) => {
                if self.popup.set_input_focus() {
                    self.focus = Focus::Input;
                }
                if let Some(new_char) = start_char {
                    self.popup.insert_input_char(new_char);
                }
            }
            InputAction::Unfocus => {
                self.popup.leave_input();
                self.preview_theme_selection();
                self.focus = Focus::Popup;
            }
            InputAction::Lock => {
                self.popup.lock_input();
                self.focus = Focus::Popup;
            }
            InputAction::Success(msg) => {
                if self.popup.set_input_success(&msg) {
                    self.focus = Focus::Input;
                }
            }
            InputAction::AutoComplete => {
                self.popup.set_input_autocomplete();
            }
            InputAction::Char(new_char) => {
                self.popup.insert_input_char(new_char);
                self.preview_theme_selection();
            }
            InputAction::Delete => {
                self.popup.delete_input_char();
                self.preview_theme_selection();
            }
            InputAction::CursorLeft => {
                if !self.switch_popup_target(false) {
                    self.popup.move_cursor_left();
                }
            }
            InputAction::CursorRight => {
                if !self.switch_popup_target(true) {
                    self.popup.move_cursor_right();
                }
            }
            InputAction::Enter => self.on_input_enter(),
            InputAction::Paste(data) => self.on_input_paste(data),
            InputAction::PasteClipboard => {
                match Clipboard::new().and_then(|mut cb| cb.get_text()) {
                    Ok(data) => self.on_input_paste(data),
                    Err(e) => error!("{}", e),
                }
            }
            InputAction::Error(msg) => {
                if (self.popup.can_close()) && self.popup.invalidate_input(&msg) {
                    self.focus = Focus::Input;
                }
            }
        }
    }

    /// Shows the commands of the next or the previous chain, validator or collator while the
    /// commands of one are displayed and nothing was typed yet.
    fn switch_popup_target(&mut self, down: bool) -> bool {
        if !self.popup.is_input_empty() {
            return false;
        }

        let shown = if self.popup.is_chain_menu() && self.section == Section::Chains {
            self.move_chain_selection(down);
            self.show_chain_popup()
        } else if self.popup.is_validator_menu()
            && self.data_section() == Section::Validators
            && self.validators.is_active()
        {
            self.move_validator_selection(down);
            let shown = self.show_validator_popup();
            if !shown {
                // Keep the selection on the validator the commands belong to
                // TODO: If there are other validators with proxy further down the list it should select
                // the next one with proxy
                self.move_validator_selection(!down);
            }
            shown
        } else if self.popup.is_collator_menu()
            && self.data_section() == Section::Collators
            && self.collators.is_active()
        {
            self.move_collator_selection(down);
            let shown = self.show_collator_popup();
            if !shown {
                self.move_collator_selection(!down);
            }
            shown
        } else {
            false
        };

        if !shown {
            return false;
        }
        // Showing the commands resets the input, so the focus is set again as it was
        let was_focused = self.focus == Focus::Input;
        if was_focused && self.popup.set_input_focus() {
            self.focus = Focus::Input;
        } else {
            self.focus = Focus::Popup;
        }
        true
    }

    fn move_chain_selection(&mut self, down: bool) {
        if down {
            self.chains.move_down();
        } else {
            self.chains.move_up();
        }
    }

    fn move_validator_selection(&mut self, down: bool) {
        if down {
            self.validators.move_down();
        } else {
            self.validators.move_up();
        }
    }

    fn move_collator_selection(&mut self, down: bool) {
        if down {
            self.collators.move_down();
        } else {
            self.collators.move_up();
        }
    }

    fn handle_filter_actions(&mut self, action: FilterAction) {
        match action {
            FilterAction::Open => self.open_filter(),
            FilterAction::Start(new_char) => {
                // Only when the tabs of the main window are selected, e.g. not from a section
                if self.section.is_none() {
                    self.open_filter();
                    if self.focus == Focus::Filter {
                        self.edit_filter(|input| input.insert_char(new_char));
                    }
                }
            }
            FilterAction::Char(new_char) => self.edit_filter(|input| input.insert_char(new_char)),
            FilterAction::Delete => self.edit_filter(|input| input.delete_char()),
            FilterAction::CursorLeft => self.edit_filter(|input| input.move_cursor_left()),
            FilterAction::CursorRight => self.edit_filter(|input| input.move_cursor_right()),
            FilterAction::Paste(data) => self.edit_filter(|input| input.paste_data(data)),
            FilterAction::Select => {
                self.edit_filter(|input| input.unfocus());
                self.focus = Focus::Main;
                // The lists get the focus back, with the first row that matches highlighted
                self.update_active_section();
                match self.window {
                    Window::Validators => self.validators.select_first_match(),
                    Window::Collators => self.collators.select_first_match(),
                    _ => {}
                }
            }
            FilterAction::Clear => {
                self.edit_filter(|input| input.reset_as_filter());
                self.focus = Focus::Main;
                self.update_active_section();
            }
        }
    }

    /// Starts typing the filter of the validators or the collators displayed. The rows are
    /// selected from the main window, where every validator or collator can be selected.
    fn open_filter(&mut self) {
        if self.popup.is_visible() {
            return;
        }
        if !self.window.is_validators_selected() && !self.window.is_collators_selected() {
            return;
        }
        self.set_section(Section::None);
        self.edit_filter(|input| {
            input.set_focus();
        });
        self.focus = Focus::Filter;

        // No row is highlighted while the filter is being typed
        match self.window {
            Window::Validators => self.validators.set_focus(ListFocus::Filter),
            Window::Collators => self.collators.set_focus(ListFocus::Filter),
            _ => {}
        }
    }

    /// Edits the filter of the validators or the collators displayed.
    fn edit_filter(&mut self, edit: impl FnOnce(&mut InputFieldWidget)) {
        match self.window {
            Window::Validators => self.validators.edit_filter(edit),
            Window::Collators => self.collators.edit_filter(edit),
            _ => {}
        }
    }

    /// Whether the validators or the collators displayed are filtered.
    pub fn is_filtered(&self) -> bool {
        match self.window {
            Window::Validators => self.validators.is_filtered(),
            Window::Collators => self.collators.is_filtered(),
            _ => false,
        }
    }

    fn handle_chain_actions(&mut self, action: ChainAction) {
        match action {
            ChainAction::UpdateConnectionState(chain_key, connection_state) => {
                self.on_connection_state(chain_key, connection_state);
            }
            ChainAction::UpdateBestBlock(chain_key, block_number) => {
                self.chains.update_best_block(&chain_key, block_number);
            }
            ChainAction::UpdateFinalizedBlock(chain_key, block_number, block_hash) => {
                if self
                    .chains
                    .update_finalized_block(&chain_key, block_number, block_hash)
                {
                    self.on_finalized_block(chain_key, block_hash);
                }
            }
            ChainAction::UpdateEra(chain_key, era) => {
                let era_index = era.index();
                if self.chains.update_era(&chain_key, era) {
                    self.on_era_changed(chain_key, era_index);
                }
            }
            ChainAction::UpdateEpoch(chain_key, epoch) => {
                if self.chains.update_epoch(&chain_key, epoch) {
                    self.on_epoch_changed(chain_key);
                }
            }
            ChainAction::UpdateActiveValidators(chain_key, count) => {
                self.chains.update_active_validators(&chain_key, count);
            }
            ChainAction::UpdateTotalValidators(chain_key, count) => {
                self.chains.update_total_validators(&chain_key, count);
            }
            ChainAction::UpdateActiveNominators(chain_key, count) => {
                self.chains.update_active_nominators(&chain_key, count);
            }
            ChainAction::UpdateTotalNominators(chain_key, count) => {
                self.chains.update_total_nominators(&chain_key, count);
            }
            ChainAction::UpdateTotalStaked(chain_key, value) => {
                self.chains.update_total_staked(&chain_key, value);
            }
            ChainAction::UpdateCurrentSlot(chain_key, block_number, slot) => {
                self.chains
                    .update_current_slot(&chain_key, block_number, slot);
            }
            ChainAction::UpdateSessionIndex(chain_key, index) => {
                if self.chains.update_session_index(&chain_key, index) {
                    self.on_session_index_changed(chain_key);
                }
            }
            ChainAction::UpdateSlotDuration(chain_key, duration_ms) => {
                self.chains
                    .update_aura_slot_duration_ms(&chain_key, duration_ms);
            }
        }
    }

    /// Updates the connection state of the chain, and fetches what the chain holds once it is
    /// connected. The subscriptions that dropped are subscribed again.
    fn on_connection_state(&mut self, chain_key: SupportedRuntime, state: ConnectionState) {
        let is_updated = self
            .chains
            .update_connection_state(&chain_key, state.clone());

        match (is_updated, state) {
            (true, ConnectionState::Connected) => self.on_chain_connected(chain_key),
            (true, ConnectionState::BestBlockSubcriptionDropped) => {
                self.chains
                    .update_connection_state(&chain_key, ConnectionState::Offline);
                if let Some(chain) = self.chains.get_chain_by_runtime(chain_key) {
                    self.chains.subscribe_best_block(&chain, self.tx.clone());
                }
            }
            (true, ConnectionState::FinalizedSubscriptionDropped) => {
                self.chains
                    .update_connection_state(&chain_key, ConnectionState::Offline);
                if let Some(chain) = self.chains.get_chain_by_runtime(chain_key) {
                    self.chains
                        .subscribe_finalized_block(&chain, self.tx.clone());
                }
            }
            (_, ConnectionState::Error(e)) => {
                error!("{}", e);
            }
            _ => {}
        }
    }

    /// Fetches what the chain holds once it is connected.
    fn on_chain_connected(&mut self, runtime: SupportedRuntime) {
        if runtime.is_relay_chain() {
            self.on_relay_chain_connected(runtime);
        } else if runtime.is_asset_hub() {
            self.on_asset_hub_connected(runtime);
        } else if runtime.is_para_chain() {
            self.on_system_chain_connected(runtime);
        }
    }

    // NOTE: The data that takes a fetch per validator is only fetched for the tracked ones, and
    // the visible ones for the tracked and visible keys. Only the identity is fetched for every
    // validator, see `fetch_missing_identities`.
    fn on_relay_chain_connected(&mut self, runtime: SupportedRuntime) {
        self.fetch_validators_status(runtime);

        let keys = self
            .validators
            .get_tracked_and_visible_keys_by_runtime(runtime);
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_epoch_data(api_at, runtime, tx);
            sync::spawn_fetch_validators_session_keys(api_at, runtime, &keys, tx);
        });
    }

    fn on_asset_hub_connected(&mut self, runtime: SupportedRuntime) {
        let tracked_validator_keys = self
            .validators
            .get_tracked_validator_keys_by_runtime(runtime);
        let collator_keys = self
            .collators
            .get_tracked_and_visible_keys_by_runtime(runtime);
        let nominator_keys = self.nominators.keys_by_runtime(runtime);
        let is_all_validators_visible = CONFIG.features().all_validators_visible();

        if self.chains.get_api_and_block_hash(runtime).is_none() {
            return;
        }
        self.fetch_collators_chain_data(runtime, collator_keys.clone());
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_nominators_on_connect(api_at, runtime, &nominator_keys, tx);
            sync::spawn_fetch_validators_on_connect(
                api_at,
                runtime,
                &tracked_validator_keys,
                is_all_validators_visible,
                tx,
            );
            sync::spawn_fetch_collators_on_connect(api_at, runtime, &collator_keys, tx);
        });
    }

    /// The system chains that only hold collators.
    fn on_system_chain_connected(&mut self, runtime: SupportedRuntime) {
        let collator_keys = self
            .collators
            .get_tracked_and_visible_keys_by_runtime(runtime);

        if self.chains.get_api_and_block_hash(runtime).is_none() {
            return;
        }
        self.fetch_collators_chain_data(runtime, collator_keys.clone());
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_collators_on_connect(api_at, runtime, &collator_keys, tx);
        });
    }

    /// Fetches the data to be synced at every finalized block, e.g. all validators points.
    fn on_finalized_block(&mut self, runtime: SupportedRuntime, block_hash: BlockHash) {
        if !runtime.is_relay_chain() {
            return;
        }
        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };
        let keys = self
            .validators
            .get_tracked_and_visible_keys_by_runtime(runtime);
        self.spawn_at_block(chain.client().clone(), block_hash, move |api_at, tx| {
            sync::spawn_fetch_validators_points(api_at, runtime, &keys, tx)
        });
    }

    /// Fetches the data to be synced whenever the era changes.
    fn on_era_changed(&mut self, runtime: SupportedRuntime, era_index: u32) {
        if !runtime.is_asset_hub() {
            return;
        }

        let tracked_validator_keys = self
            .validators
            .get_tracked_validator_keys_by_runtime(runtime);
        let tracked_and_visible_validator_keys = self
            .validators
            .get_tracked_and_visible_keys_by_runtime(runtime);
        let is_all_validators_visible = CONFIG.features().all_validators_visible();

        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_validators_on_era(
                api_at,
                runtime,
                era_index,
                &tracked_validator_keys,
                &tracked_and_visible_validator_keys,
                is_all_validators_visible,
                tx,
            );
        });
    }

    /// Fetches the data to be synced whenever the session changes.
    fn on_epoch_changed(&mut self, runtime: SupportedRuntime) {
        if runtime.is_relay_chain() {
            self.fetch_validators_status(runtime);

            let keys = self
                .validators
                .get_tracked_and_visible_keys_by_runtime(runtime);
            self.spawn_at_latest_block(runtime, move |api_at, tx| {
                sync::spawn_fetch_validators_session_keys(api_at, runtime, &keys, tx);
            });

            // Aura's expected block time is derived from this relay chain's
            // epoch data, so only fetch the collators last authored now that
            // the epoch has just been processed above.
            for para_runtime in [
                runtime.asset_hub_runtime(),
                runtime.people_runtime(),
                runtime.bridge_hub_runtime(),
                runtime.coretime_runtime(),
                runtime.collectives_runtime(),
                runtime.bulletin_runtime(),
            ]
            .into_iter()
            .flatten()
            {
                let collator_keys = self
                    .collators
                    .get_tracked_and_visible_keys_by_runtime(para_runtime);
                self.spawn_at_latest_block(para_runtime, move |api_at, tx| {
                    sync::spawn_fetch_collators_last_authored_block(
                        api_at,
                        para_runtime,
                        &collator_keys,
                        tx,
                    );
                });
            }
        } else if runtime.is_asset_hub() {
            self.spawn_at_latest_block(runtime, move |api_at, tx| {
                sync::spawn_fetch_validators_totals(api_at, runtime, tx);
            });
        }
    }

    /// Fetches the collators sets and session keys of the system chains whenever their session
    /// changes.
    fn on_session_index_changed(&mut self, runtime: SupportedRuntime) {
        if !runtime.is_para_chain() {
            return;
        }
        let collator_keys = self
            .collators
            .get_tracked_and_visible_keys_by_runtime(runtime);
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_collators_on_session(api_at, runtime, &collator_keys, tx);
        });
    }

    /// Runs `fetch` with the `api` at the block `block_hash`, on its own task. The `fetch` is
    /// given the client at the block, and the sender of the actions, e.g. to spawn the fetches.
    fn spawn_at_block(
        &self,
        api: OnlineClient<CustomConfig>,
        block_hash: BlockHash,
        fetch: impl FnOnce(&OnlineClientAtBlock<CustomConfig>, &mpsc::UnboundedSender<Action>)
            + Send
            + 'static,
    ) {
        let tx = self.tx.clone();
        tokio::spawn(async move {
            match api.at_block(block_hash).await.boxed() {
                Ok(api_at) => fetch(&api_at, &tx),
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_block: {}",
                        e
                    ))));
                }
            }
        });
    }

    /// Runs `fetch` at the latest block of the chain `runtime`, see `spawn_at_block`. Returns
    /// false when the chain is not connected, and nothing was run.
    fn spawn_at_latest_block(
        &self,
        runtime: SupportedRuntime,
        fetch: impl FnOnce(&OnlineClientAtBlock<CustomConfig>, &mpsc::UnboundedSender<Action>)
            + Send
            + 'static,
    ) -> bool {
        let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) else {
            return false;
        };
        self.spawn_at_block(api, block_hash, fetch);
        true
    }

    /// Fetches the authority status, active or waiting, of the `keys` validators of the relay
    /// chain `runtime`.
    fn fetch_validators_authority_status(&self, runtime: SupportedRuntime, keys: Vec<AccountKey>) {
        if keys.is_empty() {
            return;
        }
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_validators_authority_status(api_at, runtime, &keys, tx)
        });
    }

    /// Fetches the authority status of all the validators of the relay chain `runtime`. It
    /// takes the same reads of the session set as the status of only one, so it is fetched for
    /// all of them when the chain connects, once the session changes, and when validators are
    /// added.
    fn fetch_validators_status(&self, runtime: SupportedRuntime) {
        let keys = self.validators.get_validator_keys_by_runtime(runtime);
        self.fetch_validators_authority_status(runtime, keys);
    }

    /// Fetches the data of the validators only found on-chain that became visible. It runs on
    /// every tick, an event the app already handles, since what is visible changes with the
    /// selection, the scroll, the size of the terminal or the window displayed.
    fn fetch_visible_validators_data(&mut self) {
        if !CONFIG.features().all_validators_visible() {
            return;
        }

        for (runtime, keys) in self.validators.take_unrequested_visible() {
            self.fetch_validators_data(runtime, keys);
        }
    }

    /// Fetches the identity of the `keys` validators of the relay chain `runtime`, held on the
    /// People chain. Returns false when it is not connected yet, and nothing was requested.
    fn fetch_validators_identity(&self, runtime: SupportedRuntime, keys: &[AccountKey]) -> bool {
        let Some(people_runtime) = runtime.people_runtime() else {
            return false;
        };
        let Some((api, block_hash)) = self.chains.get_api_and_block_hash(people_runtime) else {
            return false;
        };

        let keys = keys.to_vec();

        let tx = self.tx.clone();
        tokio::spawn(async move {
            let api_at = match api.at_block(block_hash).await.boxed() {
                Ok(api_at) => api_at,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_block: {}",
                        e
                    ))));
                    return;
                }
            };

            sync::spawn_fetch_validators_identity(&api_at, people_runtime, &keys, &tx);
        });
        true
    }

    /// Fetches the data of the `keys` validators of the relay chain `runtime` that takes a fetch
    /// per validator, for the ones only found on-chain that are visible: the authority status, the
    /// session keys, the bonded stake (the ledger), the payee, the stake overview and the era points.
    /// The rest, e.g. the prefs or the balance, is only fetched for the tracked ones.
    fn fetch_validators_data(&self, runtime: SupportedRuntime, keys: Vec<AccountKey>) {
        if keys.is_empty() {
            return;
        }

        // The authority status and the session keys are held on the relay chain
        if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) {
            let keys = keys.clone();
            let tx = self.tx.clone();
            tokio::spawn(async move {
                let api_at = match api.at_block(block_hash).await.boxed() {
                    Ok(api_at) => api_at,
                    Err(e) => {
                        let _ = tx.send(Action::System(SystemAction::Error(format!(
                            "Failed to client at_block: {}",
                            e
                        ))));
                        return;
                    }
                };

                sync::spawn_fetch_validators_authority_status(&api_at, runtime, &keys, &tx);
                sync::spawn_fetch_validators_session_keys(&api_at, runtime, &keys, &tx);
            });
        }

        // The staking data is held on the Asset Hub chain: the bonded stake (the ledger) and the
        // payee, and, for the active era, the stake overview and the era points. If the era is
        // not known yet, the last two are fetched for every validator once it is.
        if let Some(asset_hub_runtime) = runtime.asset_hub_runtime() {
            let chain = self.chains.get_chain_by_runtime(asset_hub_runtime);
            let era_index = chain
                .as_ref()
                .and_then(|c| c.era().as_ref())
                .map(|e| e.index());
            if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(asset_hub_runtime) {
                let tx = self.tx.clone();
                tokio::spawn(async move {
                    let api_at = match api.at_block(block_hash).await.boxed() {
                        Ok(api_at) => api_at,
                        Err(e) => {
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "Failed to client at_block: {}",
                                e
                            ))));
                            return;
                        }
                    };

                    sync::spawn_fetch_validators_staking_ledger(
                        &api_at,
                        asset_hub_runtime,
                        &keys,
                        &tx,
                    );
                    sync::spawn_fetch_validators_payee(&api_at, asset_hub_runtime, &keys, &tx);

                    if let Some(era_index) = era_index {
                        sync::spawn_fetch_validators_stake_overview(
                            &api_at,
                            asset_hub_runtime,
                            era_index,
                            &keys,
                            &tx,
                        );
                        sync::spawn_fetch_validators_era_points(
                            &api_at,
                            asset_hub_runtime,
                            era_index,
                            &keys,
                            &tx,
                        );
                    }
                });
            }
        }
    }

    fn handle_validator_actions(&mut self, action: ValidatorAction) {
        match action {
            ValidatorAction::UpdateAllValidators(runtime, stashes) => {
                if !CONFIG.features().all_validators_visible() {
                    return;
                }
                let runtime = runtime.relay_chain();
                let added = self.validators.update_all_validators(runtime, &stashes);
                if !added.is_empty() {
                    self.fetch_validators_status(runtime);
                }
            }
            ValidatorAction::UpdateValidatorPrefs(validator_key, prefs) => {
                self.validators.update_prefs(&validator_key, prefs);
            }
            ValidatorAction::UpdateValidatorPrefsNext(validator_key, prefs) => {
                self.validators.update_prefs_next(&validator_key, prefs);
            }
            ValidatorAction::UpdatePoints(validator_key, points) => {
                self.validators.update_points(&validator_key, points);
            }
            ValidatorAction::UpdateEraPoints(validator_key, points) => {
                self.validators.update_era_points(&validator_key, points);
            }
            ValidatorAction::UpdateIdentity(validator_key, identity) => {
                self.validators.update_identity(&validator_key, identity);
            }
            ValidatorAction::UpdateStakeOverview(validator_key, data) => {
                self.validators.update_stake_overview(&validator_key, data);
            }
            ValidatorAction::UpdateStakeLedger(validator_key, data) => {
                self.validators.update_stake_ledger(&validator_key, data);
            }
            ValidatorAction::UpdatePayee(validator_key, data) => {
                self.validators.update_payee(&validator_key, data);
            }
            ValidatorAction::UpdateNextKeys(validator_key, data) => {
                self.validators.update_next_keys(&validator_key, data);
            }
            ValidatorAction::UpdateQueuedKeys(validator_key, data) => {
                self.validators.update_queued_keys(&validator_key, data);
            }
            ValidatorAction::UpdateStatus(validator_key, status) => {
                self.validators.update_status(&validator_key, status);
            }
            ValidatorAction::AddAmountToStakeLedger(validator_key, amount) => {
                self.validators
                    .add_amount_to_stake_ledger(&validator_key, amount);
                // NOTE: spawn_fetch_validator_staking_ledger is called here due to the
                // 'Bonded' event that is also raised when 'Rebond' extrinsic is triggered.
                // In case of 'Rebond' funds should be added to staking_ledger and also
                // subtracted from unlocking vec.
                // To keep things in sync, a fetch for this respective stash is being called here.
                // And also a fetch to the account balance is being called here.
                let runtime = validator_key
                    .runtime()
                    .asset_hub_runtime()
                    .expect("every relay has an AssetHub chain");
                if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) {
                    let validator_keys = vec![validator_key];
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        let api_at = match api.at_block(block_hash).await.boxed() {
                            Ok(api_at) => api_at,
                            Err(e) => {
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to client at_block: {}",
                                    e
                                ))));
                                return;
                            }
                        };

                        sync::spawn_fetch_validators_staking_ledger(
                            &api_at,
                            runtime,
                            &validator_keys,
                            &tx,
                        );
                        sync::spawn_fetch_validators_account_balance(
                            &api_at,
                            runtime,
                            &validator_keys,
                            &tx,
                        );
                    });
                }
            }
            ValidatorAction::SubChunkFromStakeLedger(validator_key, chunk) => {
                self.validators
                    .sub_chunk_from_stake_ledger(&validator_key, chunk);
                // NOTE: Fetch account balance after sub-chunk to keep balance in sync.
                let runtime = validator_key
                    .runtime()
                    .asset_hub_runtime()
                    .expect("every relay has an AssetHub chain");
                if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) {
                    let validator_keys = vec![validator_key];
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        let api_at = match api.at_block(block_hash).await.boxed() {
                            Ok(api_at) => api_at,
                            Err(e) => {
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to client at_block: {}",
                                    e
                                ))));
                                return;
                            }
                        };

                        sync::spawn_fetch_validators_account_balance(
                            &api_at,
                            runtime,
                            &validator_keys,
                            &tx,
                        );
                    });
                }
            }
            ValidatorAction::AddProxy(validator_key, proxy) => {
                self.validators.add_proxy(&validator_key, proxy);
                self.refresh_proxies();
            }
            ValidatorAction::UpdateBalance(validator_key, balance) => {
                self.validators.update_balance(&validator_key, balance);
            }
            ValidatorAction::AddAmountToBalance(validator_key, amount) => {
                self.validators
                    .add_amount_to_balance(&validator_key, amount);
                // NOTE: spawn_fetch_validator_staking_ledger is called here due to the
                // 'Withdrawn' event. The witdrawn amount is expected to be available as free in balance
                // and also unlocked from `staking.ledger`. Rather than unlocking manually is easier just
                // to fetch storage for staking ledger.
                let runtime = validator_key
                    .runtime()
                    .asset_hub_runtime()
                    .expect("every relay has an AssetHub chain");
                if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) {
                    let validator_keys = vec![validator_key];
                    let tx = self.tx.clone();
                    tokio::spawn(async move {
                        let api_at = match api.at_block(block_hash).await.boxed() {
                            Ok(api_at) => api_at,
                            Err(e) => {
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to client at_block: {}",
                                    e
                                ))));
                                return;
                            }
                        };

                        sync::spawn_fetch_validators_staking_ledger(
                            &api_at,
                            runtime,
                            &validator_keys,
                            &tx,
                        );
                    });
                }
            }
        }
    }

    /// Applies an update of the on-chain collator sets of `runtime`, e.g. the authorities, to the
    /// collators and to the chain, and then syncs the on-chain only collators.
    ///
    /// The order matters:
    /// 1. The collators are updated first. On-chain only ones are added there before their
    ///    status is set, so the new ones get a status in the same step.
    /// 2. The chain's aura state is updated before syncing, since the collators to keep are
    ///    taken from it. Otherwise the ones just added would not be in it and get pruned.
    fn update_collator_set(
        &mut self,
        runtime: SupportedRuntime,
        update_collators: impl FnOnce(&mut CollatorsList, bool),
        update_chains: impl FnOnce(&mut ChainsList),
    ) {
        let is_all_collators_visible = CONFIG.features().all_collators_visible();
        update_collators(&mut self.collators, is_all_collators_visible);
        update_chains(&mut self.chains);
        self.sync_onchain_collators(runtime);
    }

    /// Once the on-chain sets of `runtime` were updated, drops the untracked collators that left
    /// them. The data of the ones added is fetched once they are visible.
    fn sync_onchain_collators(&mut self, runtime: SupportedRuntime) {
        if !CONFIG.features().all_collators_visible() {
            return;
        }

        let chain = self.chains.get_chain_by_runtime(runtime);
        if let Some(aura) = chain.as_ref().and_then(|chain| chain.aura().as_ref()) {
            let keep: HashSet<[u8; 32]> = aura
                .authorities()
                .iter()
                .copied()
                .chain(aura.invulnerables().iter().copied())
                .chain(aura.candidates().iter().map(|(who, _)| *who))
                .collect();
            self.collators.prune_untracked(runtime, &keep);
        }
    }

    /// Fetches the data of the collators only found on-chain that became visible. It runs on
    /// every tick, an event the app already handles, since what is visible changes with the
    /// selection, the scroll, the size of the terminal or the window displayed.
    fn fetch_visible_collators_data(&mut self) {
        if !CONFIG.features().all_collators_visible() {
            return;
        }

        for (runtime, keys) in self.collators.take_unrequested_visible() {
            self.fetch_collators_chain_data(runtime, keys);
        }
    }

    /// Fetches the validators nominated by the nominators with the `keys` of the Asset Hub `runtime`.
    fn fetch_nominations(&self, runtime: SupportedRuntime, keys: Vec<AccountKey>) {
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_nominators_on_connect(api_at, runtime, &keys, tx);
        });
    }

    /// Fetches the balance, the session keys and the last authored block of the `keys` collators,
    /// all of them held on the collator's own chain.
    fn fetch_collators_chain_data(&self, runtime: SupportedRuntime, keys: Vec<AccountKey>) {
        if keys.is_empty() {
            return;
        }

        let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) else {
            return;
        };

        // Aura's expected block time is derived from the relay chain epoch data
        let is_epoch_known = self
            .chains
            .get_chain_by_runtime(runtime.relay_chain())
            .is_some_and(|chain| chain.epoch().is_some());
        let tx = self.tx.clone();
        tokio::spawn(async move {
            let api_at = match api.at_block(block_hash).await.boxed() {
                Ok(api_at) => api_at,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_block: {}",
                        e
                    ))));
                    return;
                }
            };

            sync::spawn_fetch_collators_account_balance(&api_at, runtime, &keys, &tx);
            sync::spawn_fetch_collators_session_keys(&api_at, runtime, &keys, &tx);
            if is_epoch_known {
                sync::spawn_fetch_collators_last_authored_block(&api_at, runtime, &keys, &tx);
            }
        });
    }

    /// Fetches the identity of the `keys` collators of the chains of the relay chain `runtime`,
    /// held on its People chain. Returns false when it is not connected yet, and nothing was
    /// requested.
    fn fetch_collators_identity(&self, runtime: SupportedRuntime, keys: &[AccountKey]) -> bool {
        let Some(people_runtime) = runtime.relay_chain().people_runtime() else {
            return false;
        };
        let Some((api, block_hash)) = self.chains.get_api_and_block_hash(people_runtime) else {
            return false;
        };

        // The same stash can be a collator of several chains, and the identity is the same, so
        // it is only fetched once
        let mut seen = HashSet::new();
        let keys: Vec<AccountKey> = keys
            .iter()
            .filter(|key| seen.insert(key.bytes))
            .cloned()
            .collect();

        let tx = self.tx.clone();
        tokio::spawn(async move {
            let api_at = match api.at_block(block_hash).await.boxed() {
                Ok(api_at) => api_at,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_block: {}",
                        e
                    ))));
                    return;
                }
            };

            sync::spawn_fetch_collators_identity(&api_at, people_runtime, &keys, &tx);
        });
        true
    }

    fn handle_nominator_actions(&mut self, action: NominatorAction) {
        match action {
            NominatorAction::UpdateNominations(nominator_key, nominations) => {
                self.nominators
                    .update_nominations(&nominator_key, nominations);
                self.update_nominated_validators(nominator_key.runtime().relay_chain());
            }
        }
    }

    /// Lists the validators nominated by the nominators on the validators list,
    /// and fetches the data of the ones not listed before.
    fn update_nominated_validators(&mut self, runtime: SupportedRuntime) {
        if !CONFIG.features().validators_enabled() {
            return;
        }

        let targets = self.nominators.targets_by_relay_chain(runtime);
        let keep_onchain = CONFIG.features().all_validators_visible();
        let added = self
            .validators
            .update_nominated(runtime, &targets, keep_onchain);
        if added.is_empty() {
            return;
        }

        // The data of a validator listed after the start, the same as the one pinned
        self.fetch_validators_data(runtime, added.clone());
        if let Some(asset_hub_runtime) = runtime.asset_hub_runtime() {
            self.spawn_at_latest_block(asset_hub_runtime, move |api_at, tx| {
                sync::spawn_fetch_validators_on_connect(
                    api_at,
                    asset_hub_runtime,
                    &added,
                    false,
                    tx,
                );
            });
        }
    }

    fn handle_collator_actions(&mut self, action: CollatorAction) {
        match action {
            CollatorAction::UpdateAuraAuthorities(runtime, authorities) => {
                self.update_collator_set(
                    runtime,
                    |collators, is_all_collators_visible| {
                        collators.update_authorities(
                            runtime,
                            &authorities,
                            is_all_collators_visible,
                        )
                    },
                    |chains| {
                        chains.update_aura_authorities(&runtime, authorities.clone());
                    },
                );
            }
            CollatorAction::UpdateInvulnerables(runtime, invulnerables) => {
                self.update_collator_set(
                    runtime,
                    |collators, is_all_collators_visible| {
                        collators.update_invulnerables(
                            runtime,
                            &invulnerables,
                            is_all_collators_visible,
                        )
                    },
                    |chains| {
                        chains.update_aura_invulnerables(&runtime, invulnerables.clone());
                    },
                );
            }
            CollatorAction::UpdateCandidates(runtime, candidates) => {
                self.update_collator_set(
                    runtime,
                    |collators, is_all_collators_visible| {
                        collators.update_candidates(runtime, &candidates, is_all_collators_visible)
                    },
                    |chains| {
                        chains.update_aura_candidates(&runtime, candidates.clone());
                    },
                );
            }
            CollatorAction::UpdateAuthoredBlock(runtime, block_number, slot) => {
                if let Some(chain) = self.chains.get_chain_by_runtime(runtime) {
                    if let Some(aura) = &chain.aura() {
                        self.collators.update_authored_block(
                            runtime,
                            aura.authorities(),
                            block_number,
                            slot,
                        );
                    }
                }
            }
            CollatorAction::UpdateLastAuthoredBlock(collator_key, block_number) => {
                let runtime = collator_key.runtime();
                // Calculate the expected Aura block time and update the chain. This is
                // the minimum per-block cadence the relay chain can absorb, per the
                // runtime source: `RELAY_CHAIN_SLOT_DURATION_MILLIS / BLOCK_PROCESSING_VELOCITY`.
                if let Some(rc_chain) = self.chains.get_chain_by_runtime(runtime.relay_chain()) {
                    let block_time_ms = rc_chain
                        .epoch()
                        .as_ref()
                        .map(|e| e.block_time_ms())
                        .unwrap_or(0);

                    let expected_aura_block_time_ms =
                        (block_time_ms / runtime.block_processing_velocity()) * 1000;

                    self.chains
                        .update_aura_block_time_ms(&runtime, expected_aura_block_time_ms);
                }

                if let Some(chain) = self.chains.get_chain_by_runtime(runtime) {
                    if let Some(aura) = &chain.aura() {
                        self.collators.update_last_authored_block(
                            &collator_key,
                            block_number,
                            chain.finalized_block(),
                            aura.block_time_ms(),
                        );
                    }
                }
            }
            CollatorAction::UpdateIdentity(stash_bytes, identity) => {
                self.collators.update_identity(stash_bytes, identity);
            }
            CollatorAction::UpdateNextKeys(collator_key, keys) => {
                self.collators.update_next_keys(&collator_key, keys);
            }
            CollatorAction::UpdateQueuedKeys(collator_key, keys) => {
                self.collators.update_queued_keys(&collator_key, keys);
            }
            CollatorAction::AddProxy(collator_key, proxy) => {
                self.collators.add_proxy(&collator_key, proxy);
                self.refresh_proxies();
            }
            CollatorAction::UpdateStatus(collator_key, status) => {
                self.collators.update_status(&collator_key, status);
            }
            CollatorAction::UpdateDeposit(collator_key, amount) => {
                self.collators.update_deposit(&collator_key, amount);
            }
            CollatorAction::UpdateBalance(collator_key, balance) => {
                self.collators.update_balance(&collator_key, balance);
            }
        }
    }

    fn handle_transaction_actions(&mut self, action: TxAction) {
        match action {
            TxAction::Processing => {
                self.popup.show_message("processing transaction");
                // Switch app focus to main while rendering transaction status
                self.focus = Focus::Main;
            }
            TxAction::Message(message) => {
                self.popup.update_message(message);
            }
            TxAction::InBestBlock(block_hash) => {
                let message = format!("transaction in block {block_hash}");
                info!("{message}");
                self.popup.update_message(message);
            }
            // NOTE: The following actions can just be logged.
            TxAction::InFinalizedBlock(block_hash) => {
                info!("Transaction finalized in block {block_hash}");
            }
            TxAction::Success => {
                info!("Transaction succeeded");
                self.close_popup();
            }
            TxAction::Error(err) => {
                error!("Transaction error: {}", err);
                self.close_popup();
            }
        }
    }

    fn handle_scanner_actions(&mut self, action: ScannerAction) {
        match action {
            ScannerAction::Init => {
                // Switch app focus to scanner while rendering QR Code and Webcam
                self.focus = Focus::Scanner;

                let tx = self.tx.clone();
                // The QR code is read as text, unless it is the signature of a transaction
                let is_qrcode_text = self.popup.is_scan_account_mode();
                let (ctrl_tx, mut ctrl_rx) = mpsc::unbounded_channel::<ThreadAction>();
                thread::spawn(move || {
                    let mut scanner = match Scanner::new() {
                        Ok(s) if !is_qrcode_text => s.as_signature(),
                        Ok(s) => s,
                        Err(e) => {
                            let _ = tx.send(Action::Scanner(ScannerAction::Error(e.to_string())));
                            return;
                        }
                    };
                    if let Err(e) = scanner.open() {
                        let _ = tx.send(Action::Scanner(ScannerAction::Error(e.to_string())));
                        return;
                    }
                    loop {
                        // Stop on an explicit Stop, or when the app drops the sender.
                        match ctrl_rx.try_recv() {
                            Ok(ThreadAction::Stop)
                            | Err(mpsc::error::TryRecvError::Disconnected) => break,
                            Err(mpsc::error::TryRecvError::Empty) => {}
                        }
                        match scanner.scan_frame() {
                            Ok((Some(bytes), frame)) => {
                                let _ = tx.send(Action::Scanner(ScannerAction::Decoded(bytes)));
                                let _ = tx.send(Action::Scanner(ScannerAction::Frame(frame)));
                                break;
                            }
                            Ok((None, frame)) => {
                                let _ = tx.send(Action::Scanner(ScannerAction::Frame(frame)));
                            }
                            Err(e) => {
                                let _ =
                                    tx.send(Action::Scanner(ScannerAction::Error(e.to_string())));
                            }
                        }
                        thread::sleep(Duration::from_millis(50));
                    }
                });
                self.popup.start_scanner(ctrl_tx);
            }
            ScannerAction::Decoded(bytes) => {
                if self.popup.is_confirmation_mode() {
                    self.on_qr_decoded_signature(&bytes);
                } else if self.popup.is_scan_account_mode() {
                    self.on_qr_decoded_address(&bytes);
                };
            }
            ScannerAction::Frame(frame) => {
                self.popup.update_scanner_frame(frame);
            }
            ScannerAction::Error(e) => error!("{e}"),
        }
    }

    fn handle_update_actions(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::NewVersionAvailable(version) => {
                self.new_version = Some(version);
            }
            UpdateAction::Start => {
                self.popup.show_message("starting update");
                // Switch app focus to main while rendering transaction status
                self.focus = Focus::Main;

                let tx = self.tx.clone();

                tokio::spawn(async move {
                    let res = reqwest::Client::builder()
                        .user_agent(format!("suno/{}", env!("CARGO_PKG_VERSION")))
                        .build();
                    let client = match res {
                        Ok(client) => client,
                        Err(e) => {
                            error!("{}", e);
                            let _ = tx.send(Action::Update(UpdateAction::Error));
                            return;
                        }
                    };

                    let res = update::start(&client, None).await;
                    match res {
                        Ok(release) => {
                            let _ = tx.send(Action::Update(UpdateAction::Download(release)));
                        }
                        Err(e) => {
                            error!("{}", e);
                            let _ = tx.send(Action::Update(UpdateAction::Error));
                        }
                    }
                });
            }
            UpdateAction::Download(release) => {
                self.popup
                    .update_message(format!("downloading {}", release.tag_name()));

                let tx = self.tx.clone();

                tokio::spawn(async move {
                    let res = reqwest::Client::builder()
                        .user_agent(format!("suno/{}", env!("CARGO_PKG_VERSION")))
                        .build();
                    let client = match res {
                        Ok(client) => client,
                        Err(e) => {
                            error!("{}", e);
                            let _ = tx.send(Action::Update(UpdateAction::Error));
                            return;
                        }
                    };

                    let res = update::asset_name_for_platform();
                    let asset_name = match res {
                        Ok(asset_name) => asset_name,
                        Err(e) => {
                            error!("{}", e);
                            let _ = tx.send(Action::Update(UpdateAction::Error));
                            return;
                        }
                    };

                    let res = update::download(&client, &release, &asset_name).await;
                    match res {
                        Ok((bytes, expected_hash)) => {
                            let _ = tx.send(Action::Update(UpdateAction::Validate(
                                asset_name,
                                bytes,
                                expected_hash,
                            )));
                        }
                        Err(e) => {
                            error!("{}", e);
                            let _ = tx.send(Action::Update(UpdateAction::Error));
                        }
                    }
                });
            }
            UpdateAction::Validate(asset_name, bytes, expected_hash) => {
                self.popup.update_message("validating");

                if let Err(e) = update::validate(&bytes, &expected_hash) {
                    error!("{}", e);
                    let _ = self.tx.send(Action::Update(UpdateAction::Error));
                    return;
                }

                if let Err(e) = update::extract_and_replace(&bytes, &asset_name) {
                    error!("{}", e);
                    let _ = self.tx.send(Action::Update(UpdateAction::Error));
                    return;
                }

                let _ = self.tx.send(Action::Update(UpdateAction::Complete));
            }
            UpdateAction::Complete => {
                self.popup
                    .show_upgrade_complete("upgrade complete, restart to apply");
            }
            UpdateAction::Error => {
                self.popup.show_upgrade_error();
            }
        }
    }

    /// Handles the noop event of the terminal.
    pub fn noop(&self) {}

    /// Handles the tick event of the terminal.
    pub fn tick(&mut self) {
        if self.section == Section::Chains && self.popup.get_mode() == PopupMode::Metadata {
            self.popup.advance_metadata_frame();
        }

        self.fetch_visible_validators_data();
        self.fetch_visible_collators_data();
        self.fetch_missing_identities();
    }

    /// Fetches the identity of every validator, or collator, listed in the window displayed, once
    /// the People chain is connected. The identity is critical to tell them apart, so unlike the
    /// rest of the data, which is only fetched for the tracked and the visible ones, it is
    /// fetched for all of them. The other window is fetched when it is displayed.
    /// It runs on every tick, an event the app already handles, since the validators and the
    /// collators are listed as the sets of the chains are fetched.
    fn fetch_missing_identities(&mut self) {
        match self.window {
            Window::Validators => {
                for (runtime, keys) in self.validators.get_keys_without_identity_requested() {
                    if self.fetch_validators_identity(runtime, &keys) {
                        self.validators.mark_identity_requested(&keys);
                    }
                }
            }
            Window::Collators => {
                for (runtime, keys) in self.collators.get_keys_without_identity_requested() {
                    if self.fetch_collators_identity(runtime, &keys) {
                        self.collators.mark_identity_requested(&keys);
                    }
                }
            }
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Moves row selection up.
    pub fn move_up(&mut self) {
        if self.popup.is_visible() {
            self.popup.move_up();
            self.preview_theme_selection();
            return;
        }

        // The logs have no section of their own, they scroll while the main window is selected
        if self.section.is_none() && self.window.is_logs_selected() {
            self.logs.move_up();
            return;
        }

        // The same for the help, whose text can be taller than the window
        if self.section.is_none() && self.window.is_help_selected() {
            self.help.scroll_up();
            return;
        }

        match self.data_section() {
            Section::Chains => {
                self.chains.move_up();
            }
            Section::Validators => {
                self.validators.move_up();
            }
            Section::Collators => {
                self.collators.move_up();
            }
            Section::Nominators => {
                self.nominators.move_up();
            }
            Section::Proxies => {
                self.proxies.move_up();
            }
            _ => {}
        };
    }

    /// Moves row selection down.
    pub fn move_down(&mut self) {
        if self.popup.is_visible() {
            self.popup.move_down();
            self.preview_theme_selection();
            return;
        }

        // The logs have no section of their own, they scroll while the main window is selected
        if self.section.is_none() && self.window.is_logs_selected() {
            self.logs.move_down();
            return;
        }

        // The same for the help, whose text can be taller than the window
        if self.section.is_none() && self.window.is_help_selected() {
            self.help.scroll_down();
            return;
        }

        match self.data_section() {
            Section::Chains => {
                self.chains.move_down();
            }
            Section::Validators => {
                self.validators.move_down();
            }
            Section::Collators => {
                self.collators.move_down();
            }
            Section::Nominators => {
                self.nominators.move_down();
            }
            Section::Proxies => {
                self.proxies.move_down();
            }
            _ => {}
        };
    }

    fn prev_tab(&mut self) {
        if self.section.is_none() {
            if self.window == Window::first(CONFIG.features()) {
                self.set_section(Section::Chains);
            } else {
                self.prev_window();
            }
        } else {
            self.set_section(Section::None);
        }
    }

    /// Focuses the window tabs, and when they already are, selects the next window.
    fn next_tab(&mut self) {
        if self.section.is_none() {
            self.next_window();
        } else {
            self.set_section(Section::None);
        }
    }

    /// Extends the selection up, on the logs. Anywhere else it moves the selection up.
    fn extend_up(&mut self) {
        if self.section.is_none() && self.window.is_logs_selected() && !self.popup.is_visible() {
            self.logs.extend_up();
        } else {
            self.move_up();
        }
    }

    /// Extends the selection down, on the logs. Anywhere else it moves the selection down.
    fn extend_down(&mut self) {
        if self.section.is_none() && self.window.is_logs_selected() && !self.popup.is_visible() {
            self.logs.extend_down();
        } else {
            self.move_down();
        }
    }

    /// Moves the active section up.
    pub fn section_up(&mut self) {
        let config = CONFIG.clone();
        let section = self.section.up(&config.features, !self.proxies.is_empty());
        self.set_section(section);
    }

    /// Moves the active section down.
    pub fn section_down(&mut self) {
        let config = CONFIG.clone();
        let section = self
            .section
            .down(&config.features, !self.proxies.is_empty());
        self.set_section(section);
    }

    /// The section whose rows are displayed on the main window. It is the selected section,
    /// unless the main window itself is selected, then it is the section it was reached from.
    pub fn data_section(&self) -> Section {
        match (&self.section, self.window) {
            (Section::None, Window::Validators) => Section::Validators,
            (Section::None, Window::Collators) => Section::Collators,
            (Section::None, _) => self.main_section.clone(),
            (section, _) => section.clone(),
        }
    }

    fn set_section(&mut self, section: Section) {
        if section.is_none() {
            // Keep showing the rows of the section the main window was reached from, falling
            // back to the first enabled one when coming from the chains.
            let features = CONFIG.features();
            self.main_section = match self.section {
                Section::Validators => Section::Validators,
                Section::Collators => Section::Collators,
                _ if features.validators_enabled() => Section::Validators,
                _ if features.collators_enabled() => Section::Collators,
                _ => Section::Chains,
            };
        }
        // Selecting a section brings up its window
        match section {
            Section::Validators => self.window = Window::Validators,
            Section::Collators => self.window = Window::Collators,
            _ => {}
        }
        self.section = section;
        self.update_visible_lists();
        self.update_active_section();
    }

    /// Tells the lists whether their window is the one displayed, since the data of the
    /// validators and collators only found on-chain is only fetched while they are visible.
    fn update_visible_lists(&mut self) {
        // The filter belongs to the window displayed
        if self.focus == Focus::Filter {
            self.focus = Focus::Main;
        }
        self.validators
            .set_visible(self.window.is_validators_selected());
        self.collators
            .set_visible(self.window.is_collators_selected());
    }

    fn update_active_section(&mut self) {
        let data_section = self.data_section();
        // The left panes lose the focus while the main window is selected
        let focus_of = |is_data_section: bool| match (is_data_section, &self.section) {
            (false, _) => ListFocus::None,
            (true, Section::None) => ListFocus::Main,
            (true, _) => ListFocus::Compact,
        };
        let validators_focus = focus_of(data_section == Section::Validators);
        let collators_focus = focus_of(data_section == Section::Collators);

        self.chains.set_active(self.section == Section::Chains);
        self.proxies.set_active(self.section == Section::Proxies);
        self.nominators
            .set_active(self.section == Section::Nominators);
        self.validators.set_focus(validators_focus);
        self.collators.set_focus(collators_focus);

        // Selecting a section brings back the focus to the main
        if self.focus == Focus::None {
            self.focus = Focus::Main;
        }
    }

    /// Selects the previous window.
    fn prev_window(&mut self) {
        self.window = self.window.prev(CONFIG.features());
        self.update_visible_lists();
        self.sync_section_with_window();
    }

    /// Selects the next window.
    fn next_window(&mut self) {
        self.window = self.window.next(CONFIG.features());
        self.update_visible_lists();
        self.sync_section_with_window();
    }

    /// Keeps the selected section in line with the window displayed: the validators and
    /// collators windows select their own section, unless the main window is the selected
    /// section, then the rows selected are the ones of the window displayed.
    fn sync_section_with_window(&mut self) {
        match (&self.section, self.window) {
            (Section::None, _) => self.update_active_section(),
            (_, Window::Validators) => self.set_section(Section::Validators),
            (_, Window::Collators) => self.set_section(Section::Collators),
            _ => {}
        }
    }

    /// Opens the theme selection popup.
    fn open_theme_menu(&mut self) {
        if self.popup.is_visible() {
            return;
        }

        let catalog = CONFIG.theme_catalog();
        if catalog.is_empty() {
            return;
        }

        self.theme_index_before_menu = self.theme_index;
        let active_name = catalog[self.theme_index].0.clone();

        let names: Vec<String> = catalog.into_iter().map(|(name, _)| name).collect();
        self.popup.show_theme_menu(names, active_name);

        // Dispatch focus to the input field so typing filters and up/down/enter/esc navigate.
        let _ = self.tx.send(Action::Input(InputAction::Editing(None)));
    }

    /// Enter on a popup that is not editing the input.
    fn confirm_popup(&mut self) {
        if self.popup.get_mode() == PopupMode::Features {
            self.toggle_feature();
        }
    }

    /// Adds the account of the `call`, found on the Asset Hub `runtime`. A validator is added to
    /// its relay chain, and a collator to the Asset Hub.
    fn add_account(&mut self, call: Call, runtime: SupportedRuntime) {
        match call {
            Call::AddValidator { stash: Some(stash) } => {
                let relay = runtime.relay_chain();
                if let Some(key) = self.validators.add_pinned(relay, stash) {
                    save_pinned_validators(self.validators.get_pinned());
                    self.fetch_validators_data(relay, vec![key.clone()]);
                    self.fetch_validators_proxy_status(runtime, vec![key]);
                }
            }
            Call::AddNominator { stash: Some(stash) } => {
                if let Some(key) = self.nominators.add_pinned(runtime, stash) {
                    save_pinned_nominators(self.nominators.get_pinned());
                    self.fetch_nominations(runtime, vec![key]);
                }
            }
            Call::AddCollator { stash: Some(stash) } => {
                if let Some(key) = self.collators.add_pinned(runtime, stash) {
                    save_pinned_collators(self.collators.get_pinned());
                    self.fetch_collators_chain_data(runtime, vec![key.clone()]);
                    self.fetch_collators_proxy_status(runtime, vec![key]);
                }
            }
            Call::AddProxyAccount {
                account: Some(account),
            } => {
                save_proxy_account(runtime, account);
                self.fetch_proxy_status(runtime);
                self.refresh_proxies();
            }
            Call::RemoveProxyAccount {
                account: Some(account),
            } => {
                remove_proxy_account(runtime, account);
                self.fetch_proxy_status(runtime);
                self.refresh_proxies();
            }
            _ => return,
        }
        self.close_popup();
    }

    /// Handle decoded bytes from qr code when popup is in scan account mode
    fn on_qr_decoded_address(&mut self, bytes: &[u8]) {
        let Some((call, runtime)) = self.popup.get_scan_account() else {
            return;
        };
        let account = from_utf8(bytes)
            .ok()
            .and_then(|text| AccountId32::from_str(extract_address_from_qr(text)).ok());
        match account {
            Some(account) => self.add_account(call.with_address(account), runtime),
            None => {
                error!("The QR code is not an address");
                let _ = self.tx.send(Action::Scanner(ScannerAction::Init));
            }
        }
    }

    /// Lists the proxy accounts of every chain, with the validators and the collators that hold
    /// a proxy with each of them.
    fn refresh_proxies(&mut self) {
        let mut proxies = ProxyState::list_all();
        for proxy in &mut proxies {
            for validator in self.validators.validators_iter() {
                proxy.track_validator(&validator.proxies);
            }
            for collator in self.collators.collators_iter() {
                proxy.track_collator(&collator.proxies);
            }
        }
        self.proxies.load(proxies);

        // The proxies section can't stay selected once there are no proxy accounts
        if self.section == Section::Proxies && self.proxies.is_empty() {
            self.section_up();
        }
    }

    /// Removes the proxy account selected
    fn remove_proxy_selected(&mut self) {
        let Some(proxy) = self.proxies.get_selected() else {
            return;
        };
        if !proxy.is_removable() {
            return;
        }
        remove_proxy_account(proxy.runtime, proxy.account);
        self.fetch_proxy_status(proxy.runtime);
        self.refresh_proxies();
    }

    /// Shows the features, to be switched on and off.
    fn open_features(&mut self) {
        if self.popup.is_visible() {
            return;
        }
        self.popup.show_features(CONFIG.feature_states());
        self.focus = Focus::Popup;
    }

    /// Switches the selected feature on or off.
    fn toggle_feature(&mut self) {
        if let Some(state) = self.popup.get_selected_feature() {
            self.set_feature(state.feature, !state.value);
        }
    }

    /// Sets the selected feature back to the value of the config file.
    fn reset_feature(&mut self) {
        if let Some(state) = self.popup.get_selected_feature() {
            self.set_feature(state.feature, state.file_value());
        }
    }

    /// Saves the feature, and updates what it affects right away: the columns are read when
    /// rendered, and the validators or the collators only found on-chain are removed or
    /// fetched.
    fn set_feature(&mut self, feature: Feature, value: bool) {
        save_feature(feature, value);
        self.popup.update_features(CONFIG.feature_states());

        match feature {
            Feature::ShowAllValidators => {
                if CONFIG.features().all_validators_visible() {
                    self.fetch_all_validators();
                } else {
                    self.validators.remove_on_chain();
                }
            }
            Feature::ShowAllCollators => {
                if CONFIG.features().all_collators_visible() {
                    self.fetch_all_collator_sets();
                } else {
                    self.collators.remove_on_chain();
                }
            }
            _ => {}
        }
    }

    /// Fetches the proxy status of the validators, on an Asset Hub, and of the collators of the
    /// chain `runtime`, for all its proxy accounts. The proxies known are forgotten first, since
    /// the proxy accounts changed.
    fn fetch_proxy_status(&mut self, runtime: SupportedRuntime) {
        let collator_keys = self
            .collators
            .get_tracked_and_visible_keys_by_runtime(runtime);
        self.collators.clear_proxies(runtime);

        let mut validator_keys = Vec::new();
        if runtime.is_asset_hub() {
            validator_keys = self
                .validators
                .get_tracked_validator_keys_by_runtime(runtime);
            self.validators.clear_proxies(runtime.relay_chain());
        }

        self.fetch_validators_proxy_status(runtime, validator_keys);
        self.fetch_collators_proxy_status(runtime, collator_keys);
    }

    /// Fetches the proxy status of the `keys` validators, held on the Asset Hub `runtime`, for
    /// all the signers of the chain. It checks if any of the proxy accounts available can be used.
    fn fetch_validators_proxy_status(&self, runtime: SupportedRuntime, keys: Vec<AccountKey>) {
        let proxies = runtime.signer_account_ids();
        if keys.is_empty() || proxies.is_empty() {
            return;
        }
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_validators_proxy_status(api_at, runtime, &keys, &proxies, tx);
        });
    }

    /// Fetches the proxy status of the `keys` collators of the chain `runtime`, for all the
    /// signers of the chain.
    fn fetch_collators_proxy_status(&self, runtime: SupportedRuntime, keys: Vec<AccountKey>) {
        let proxies = runtime.signer_account_ids();
        if keys.is_empty() || proxies.is_empty() {
            return;
        }
        self.spawn_at_latest_block(runtime, move |api_at, tx| {
            sync::spawn_fetch_collators_proxy_status(api_at, runtime, &keys, &proxies, tx);
        });
    }

    /// Fetches the validators found on-chain of the connected relay chains, held on their
    /// Asset Hub.
    fn fetch_all_validators(&self) {
        for chain in self.chains.chains_iter() {
            let runtime = chain.runtime();
            if !runtime.is_asset_hub() {
                continue;
            }
            self.spawn_at_latest_block(runtime, move |api_at, tx| {
                sync::spawn_fetch_all_validators(api_at, runtime, tx)
            });
        }
    }

    /// Fetches the collators found on-chain of the connected system chains.
    fn fetch_all_collator_sets(&self) {
        for chain in self.chains.chains_iter() {
            let runtime = chain.runtime();
            if !runtime.is_para_chain() {
                continue;
            }
            self.spawn_at_latest_block(runtime, move |api_at, tx| {
                sync::spawn_fetch_collator_sets(api_at, runtime, tx)
            });
        }
    }

    /// Applies the theme named `name` from `CONFIG.theme_catalog()`, if any.
    fn apply_theme_by_name(&mut self, name: &str) {
        let catalog = CONFIG.theme_catalog();
        let Some(index) = catalog.iter().position(|(n, _)| n == name) else {
            return;
        };
        self.theme_index = index;
        self.theme = catalog[index].1;
        self.chains.set_theme(self.theme);
        self.proxies.set_theme(self.theme);
        self.nominators.set_theme(self.theme);
        self.popup.set_theme(self.theme);
        self.logs.set_theme(self.theme);
        self.help.set_theme(self.theme);
        info!("Theme changed to: {}", name);
    }

    /// Live-previews the theme currently highlighted in the theme menu.
    fn preview_theme_selection(&mut self) {
        if self.popup.get_mode() != PopupMode::ThemeMenu {
            return;
        }
        if let Some(name) = self.popup.get_selected_theme() {
            self.apply_theme_by_name(&name);
        }
    }

    /// Handle enter when the popup is showing the theme menu.
    fn on_theme_menu_enter(&mut self) {
        let Some(name) = self.popup.get_confirmed_theme() else {
            return;
        };
        self.apply_theme_by_name(&name);
        save_active_theme(&name);
        self.close_popup();
    }

    /// Toggles the masked state of the application.
    fn toggle_mask(&mut self) {
        self.masked = !self.masked;
        self.validators.toggle_mask();
        self.collators.toggle_mask();
        self.popup.toggle_mask();
    }

    /// Pins the selected validator or collator, when it is not listed in the config, or unpins it.
    /// The pinned ones are kept in the prefs file.
    fn toggle_pin(&mut self) {
        if self.popup.is_visible() {
            return;
        }

        match self.window {
            Window::Validators => {
                // Toggle pin only from the main window, not from the compact list
                if self.validators.focus() != ListFocus::Main {
                    return;
                }
                let Some(key) = self.validators.get_selected_ref().map(|v| v.key().clone()) else {
                    return;
                };
                let Some(origin) = self.validators.toggle_pinned(&key) else {
                    return;
                };
                // The validators only found on-chain are only listed when all of them are
                if origin.is_onchain() && !CONFIG.features().all_validators_visible() {
                    self.validators.remove(&key);
                }
                save_pinned_validators(self.validators.get_pinned());
            }
            Window::Collators => {
                // Toggle pin only from the main window, not from the compact list
                if self.collators.focus() != ListFocus::Main {
                    return;
                }
                let Some(key) = self.collators.get_selected_ref().map(|c| c.key().clone()) else {
                    return;
                };
                let Some(origin) = self.collators.toggle_pinned(&key) else {
                    return;
                };
                // The collators only found on-chain are only listed when all of them are
                if origin.is_onchain() && !CONFIG.features().all_collators_visible() {
                    self.collators.remove(&key);
                }
                save_pinned_collators(self.collators.get_pinned());
            }
            _ => {}
        }
    }

    /// Returns the hint to show in the legend when the highlighted validator or collator is toggled,
    /// which only happens from the main window.
    pub fn toggle_hint(&self) -> Option<&'static str> {
        match self.window {
            Window::Validators if self.validators.focus() == ListFocus::Main => self
                .validators
                .get_highlighted_ref()?
                .origins
                .toggle_label(),
            Window::Collators if self.collators.focus() == ListFocus::Main => {
                self.collators.get_highlighted_ref()?.origin.toggle_label()
            }
            _ => None,
        }
    }

    /// Toggles between grouped and listed view.
    fn toggle_view(&mut self) {
        if self.data_section() == Section::Collators && self.collators.is_active() {
            self.collators.toggle_view();
        }
    }

    /// Shows the commands of the validator selected.
    fn show_validator_popup(&mut self) -> bool {
        if !self.validators.is_proxy_valid() && !self.validators.has_commands_available() {
            return false;
        }
        let Some(validator) = self.validators.get_selected() else {
            return false;
        };
        let Some(chain) = self.chains.get_chain_by_runtime(
            validator
                .runtime()
                .asset_hub_runtime()
                .expect("every relay has an Asset Hub chain"),
        ) else {
            return false;
        };
        let Some(active_era) = chain.era() else {
            return false;
        };
        self.popup
            .show_validator_commands(&validator, active_era.index());
        true
    }

    /// Shows the commands of the collator selected.
    fn show_collator_popup(&mut self) -> bool {
        if !self.collators.is_proxy_valid() && !self.collators.has_commands_available() {
            return false;
        }
        let Some(collator) = self.collators.get_selected() else {
            return false;
        };
        self.popup.show_collator_commands(&collator);
        true
    }

    /// Shows the commands of the chain selected.
    fn show_chain_popup(&mut self) -> bool {
        let Some(chain) = self.chains.get_selected() else {
            return false;
        };
        self.popup.show_chain_commands(&chain);
        true
    }

    /// Shows the commands of the proxy selected.
    fn show_proxy_popup(&mut self) -> bool {
        let Some(proxy) = self.proxies.get_selected().filter(|p| p.is_removable()) else {
            return false;
        };
        self.popup.show_proxy_commands(&proxy);
        true
    }

    /// Open popup depending on the selected pane section.
    pub fn open_popup(&mut self) {
        if self.popup.is_visible() {
            return;
        }

        if self.data_section() == Section::Validators
            && self.validators.is_active()
            && self.show_validator_popup()
        {
            let _ = self.tx.send(Action::Input(InputAction::Editing(None)));
        };

        if self.data_section() == Section::Collators
            && self.collators.is_active()
            && self.show_collator_popup()
        {
            let _ = self.tx.send(Action::Input(InputAction::Editing(None)));
        };

        if self.section == Section::Chains && self.chains.is_active() && self.show_chain_popup() {
            let _ = self.tx.send(Action::Input(InputAction::Editing(None)));
        };

        if self.section == Section::Proxies && self.proxies.is_active() && self.show_proxy_popup() {
            let _ = self.tx.send(Action::Input(InputAction::Editing(None)));
        };

        // Without anything focused, the commands of the first chain are the ones who become visible
        if self.focus == Focus::None
            && !self.popup.is_visible()
            && !self.window.is_logs_selected()
            && !self.window.is_help_selected()
        {
            self.set_section(Section::Chains);
            if self.chains.get_selected().is_none() {
                self.chains.init_table();
            }
            if self.show_chain_popup() {
                let _ = self.tx.send(Action::Input(InputAction::Editing(None)));
            }
        }
    }

    /// Open confirm and sign popup
    pub fn open_confirm_and_sign_popup(&mut self, ctx: &ConfirmationContext) {
        if !self.popup.is_visible() {
            return;
        }

        self.popup.show_confirm_and_sign(ctx);

        // If configured for QR signing, dispatch action to initialize scanner.
        // Otherwise, change focus to the input field for password signing.
        if ctx.runtime.is_qrcode_enabled() {
            let _ = self.tx.send(Action::Scanner(ScannerAction::Init));
        } else {
            let _ = self.tx.send(Action::Input(InputAction::Editing(None)));
        }
    }

    /// Build chain-specs QR code popup
    pub fn build_chain_specs_qrcode_popup(&mut self, chain: &Chain) {
        let config = CONFIG.clone();
        if !self.popup.is_visible() {
            return;
        }

        // Lock the input focus to prevent user interaction while the QR code is being built.
        let _ = self.tx.send(Action::Input(InputAction::Lock));

        let api = chain.client().clone();
        let runtime = chain.runtime();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let at_block = match api.at_current_block().await.boxed() {
                Ok(client) => client,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_current_block: {}",
                        e
                    ))));
                    return;
                }
            };

            // Sign the QR code payload based on the vault configuration, if available.
            let qr_bytes = if let Some(vault) = config.vault.as_ref() {
                let chain_specs = NetworkSpecsToSend::from(runtime);
                let payload = chain_specs.payload();
                match vault.sign(&payload) {
                    Ok((public_key, signature)) => build_chain_specs_qrcode_signed(
                        &payload,
                        public_key.as_ref(),
                        signature.as_ref(),
                    ),
                    Err(e) => {
                        let _ = tx.send(Action::System(SystemAction::Error(format!(
                            "Failed to sign chain specs: {}",
                            e
                        ))));
                        return;
                    }
                }
            } else {
                let chain_specs = NetworkSpecsToSend::from(runtime);
                let payload = chain_specs.payload();
                build_chain_specs_qrcode_unsigned(&payload)
            };

            let spec_version = at_block.spec_version();
            let ctx = Box::new(ChainSpecsContext {
                runtime,
                spec_version,
                qr_bytes,
            });
            let _ = tx.send(Action::Popup(PopupAction::ShowChainSpecsQrcode(ctx)));
        });
    }

    /// Build metatada QR code popup
    pub fn build_metadata_qrcode_popup(&mut self, chain: &Chain) {
        let config = CONFIG.clone();
        if !self.popup.is_visible() {
            return;
        }

        // Lock the input focus to prevent user interaction while the QR code is being built.
        let _ = self.tx.send(Action::Input(InputAction::Lock));

        let api = chain.client().clone();
        let runtime = chain.runtime();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            let at_block = match api.at_current_block().await.boxed() {
                Ok(client) => client,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_current_block: {}",
                        e
                    ))));
                    return;
                }
            };

            let metadata_bytes = match runtime.fetch_metadata(&at_block).await {
                Ok(bytes) => bytes,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to fetch metadata: {}",
                        e
                    ))));
                    return;
                }
            };

            let genesis_hash = runtime.chain_genesis_hash();

            // Sign the QR code payload based on the vault configuration, if available.
            let qr_bytes = if let Some(vault) = config.vault.as_ref() {
                let payload = build_metadata_payload(&metadata_bytes, &genesis_hash.into());
                match vault.sign(&payload) {
                    Ok((public_key, signature)) => build_metadata_qrcode_signed(
                        &metadata_bytes,
                        &genesis_hash.into(),
                        public_key.as_ref(),
                        signature.as_ref(),
                    ),
                    Err(e) => {
                        let _ = tx.send(Action::System(SystemAction::Error(format!(
                            "Failed to sign chain specs: {}",
                            e
                        ))));
                        return;
                    }
                }
            } else {
                build_metadata_qrcode_unsigned(&metadata_bytes, &genesis_hash.into())
            };

            let spec_version = at_block.spec_version();
            let ctx = Box::new(MetadataContext {
                runtime,
                spec_version,
                qr_bytes,
            });
            let _ = tx.send(Action::Popup(PopupAction::ShowMetadataQrcode(ctx)));
        });
    }

    /// Close menu popup
    pub fn close_popup(&mut self) {
        if !self.popup.is_visible() {
            return;
        }
        self.popup.close();
        self.focus = Focus::Main;
    }

    /// Handle input enter depending on the context
    pub fn on_input_enter(&mut self) {
        if !self.popup.is_visible() {
            return;
        }

        if self.popup.get_mode() == PopupMode::ThemeMenu {
            self.on_theme_menu_enter();
            return;
        }

        if self.data_section() == Section::Validators {
            let Some(validator) = self.validators.get_selected() else {
                return;
            };

            match self.popup.get_mode() {
                PopupMode::Menu => {
                    self.on_validator_menu_enter(validator);
                }
                PopupMode::Confirmation => {
                    self.on_validator_confirm_enter(validator);
                }
                _ => {}
            }
        };

        if self.data_section() == Section::Collators {
            let Some(collator) = self.collators.get_selected() else {
                return;
            };

            match self.popup.get_mode() {
                PopupMode::Menu => {
                    self.on_collator_menu_enter(collator);
                }
                PopupMode::Confirmation => {
                    self.on_collator_confirm_enter(collator);
                }
                _ => {}
            }
        };

        if self.section == Section::Chains {
            let Some(chain) = self.chains.get_selected() else {
                return;
            };

            if self.popup.get_mode() == PopupMode::Menu {
                self.on_chain_menu_enter(&chain);
            }
        }

        if self.section == Section::Proxies && self.popup.get_mode() == PopupMode::Menu {
            self.on_proxy_menu_enter();
        }
    }

    /// Handle enter when a proxy account is selected and popup is in menu mode, showing the
    /// commands available for it.
    fn on_proxy_menu_enter(&mut self) {
        let Some(Call::RemoveProxyAccount { .. }) = self.popup.get_input_parsed_call() else {
            return;
        };
        self.remove_proxy_selected();
        self.close_popup();
    }

    /// Handle enter when a chain is selected and popup is in menu mode
    /// showing available options
    pub fn on_chain_menu_enter(&mut self, chain: &Chain) {
        let Some(call) = self.popup.get_input_parsed_call() else {
            return;
        };

        match call {
            Call::ChainSpecs { .. } => {
                self.build_chain_specs_qrcode_popup(chain);
            }
            Call::Metadata { .. } => {
                self.build_metadata_qrcode_popup(chain);
            }
            Call::AddValidator { .. }
            | Call::AddNominator { .. }
            | Call::AddCollator { .. }
            | Call::AddProxyAccount { .. }
            | Call::RemoveProxyAccount { .. } => self.add_or_scan_account(call, chain.runtime()),
            _ => {}
        }
    }

    /// Adds the account of the `call`, or scans its address from the Vault when none was given.
    fn add_or_scan_account(&mut self, call: Call, runtime: SupportedRuntime) {
        if call.address().is_some() {
            self.add_account(call, runtime);
        } else {
            self.popup.show_scan_account(call, runtime);
            self.focus = Focus::Popup;
            let _ = self.tx.send(Action::Scanner(ScannerAction::Init));
        }
    }

    /// Handle enter when a validator is selected and popup is in menu mode
    /// showing available extrinsics/commands
    pub fn on_validator_menu_enter(&mut self, validator: Validator) {
        let Some(call) = self.popup.get_input_parsed_call() else {
            return;
        };

        match call {
            Call::Custom(custom) => self.handle_custom_command(custom, validator),
            _ => self.handle_extrinsic_calls(call, validator),
        }
    }

    pub fn handle_extrinsic_calls(&mut self, call: Call, validator: Validator) {
        let runtime = validator
            .runtime()
            .asset_hub_runtime()
            .expect("every relay has an AssetHub chain");

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };
        let api = chain.client().clone();
        let tx = self.tx.clone();
        let stash = validator.key().stash();
        let Some((proxy_account_id, supported_proxy)) = validator.get_proxy_for(runtime, &call)
        else {
            error!("No signer is defined for {}", runtime);
            return;
        };
        let proxy_identity = to_compact_string(&proxy_account_id, runtime.account_format(), 6);
        let stash_identity = validator.display_name(3);

        // Lock the input focus to prevent user interaction while the QR code is being built.
        let _ = self.tx.send(Action::Input(InputAction::Lock));

        tokio::spawn(async move {
            let at_block = match api.at_current_block().await.boxed() {
                Ok(client) => client,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_current_block: {}",
                        e
                    ))));
                    return;
                }
            };

            let call_data_bytes =
                match runtime.build_call_data(&at_block, &stash, call.clone(), supported_proxy) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        let _ = tx.send(Action::System(SystemAction::Error(format!(
                            "Failed to build call data: {}",
                            e
                        ))));
                        return;
                    }
                };

            info!("method: {}", call.to_method());
            info!("call_data: 0x{}", hex::encode(&call_data_bytes));

            let qr_bytes = match build_transaction_qrcode(
                &at_block,
                &proxy_account_id,
                &call_data_bytes,
            )
            .await
            {
                Ok(qr_bytes) => qr_bytes,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to build QR data: {}",
                        e
                    ))));
                    return;
                }
            };
            info!("qr_bytes: 0x{}", hex::encode(&qr_bytes));

            let spec_version = at_block.spec_version();
            let ctx = Box::new(ConfirmationContext {
                runtime,
                spec_version,
                proxy_identity,
                proxy_account: proxy_account_id,
                stash_identity,
                call,
                call_data_bytes,
                qr_bytes,
            });
            let _ = tx.send(Action::Popup(PopupAction::ShowConfirmAndSign(ctx)));
        });
    }

    /// Handle enter when a collator is selected and popup is in menu mode
    /// showing available extrinsics/commands
    pub fn on_collator_menu_enter(&mut self, collator: Collator) {
        let Some(call) = self.popup.get_input_parsed_call() else {
            return;
        };

        match call {
            Call::Custom(custom) => self.handle_collator_custom_command(custom, collator),
            _ => self.handle_collator_extrinsic_calls(call, collator),
        }
    }

    pub fn handle_collator_extrinsic_calls(&mut self, call: Call, collator: Collator) {
        let runtime = collator.runtime();

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };
        let api = chain.client().clone();
        let tx = self.tx.clone();
        let stash = collator.key().stash();
        let Some((proxy_account_id, supported_proxy)) = collator.get_proxy_for(runtime, &call)
        else {
            error!("No signer is defined for {}", runtime);
            return;
        };
        let proxy_identity = to_compact_string(&proxy_account_id, runtime.account_format(), 6);
        let stash_identity = collator.display_name(3);

        // Lock the input focus to prevent user interaction while the QR code is being built.
        let _ = self.tx.send(Action::Input(InputAction::Lock));

        tokio::spawn(async move {
            let at_block = match api.at_current_block().await.boxed() {
                Ok(client) => client,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to client at_current_block: {}",
                        e
                    ))));
                    return;
                }
            };

            let call_data_bytes =
                match runtime.build_call_data(&at_block, &stash, call.clone(), supported_proxy) {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        let _ = tx.send(Action::System(SystemAction::Error(format!(
                            "Failed to build call data: {}",
                            e
                        ))));
                        return;
                    }
                };

            info!("method: {}", call.to_method());
            info!("call_data: 0x{}", hex::encode(&call_data_bytes));

            let qr_bytes = match build_transaction_qrcode(
                &at_block,
                &proxy_account_id,
                &call_data_bytes,
            )
            .await
            {
                Ok(qr_bytes) => qr_bytes,
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Failed to build QR data: {}",
                        e
                    ))));
                    return;
                }
            };
            info!("qr_bytes: 0x{}", hex::encode(&qr_bytes));

            let spec_version = at_block.spec_version();
            let ctx = Box::new(ConfirmationContext {
                runtime,
                spec_version,
                proxy_identity,
                proxy_account: proxy_account_id,
                stash_identity,
                call,
                call_data_bytes,
                qr_bytes,
            });
            let _ = tx.send(Action::Popup(PopupAction::ShowConfirmAndSign(ctx)));
        });
    }

    pub fn handle_collator_custom_command(&mut self, custom: CustomCommand, collator: Collator) {
        let tx = self.tx.clone();
        let masked = self.is_masked();

        match custom.kind {
            CommandKind::Shell { run, .. } => {
                tokio::spawn(async move {
                    let run = run.replace("{stash}", &collator.key().stash().to_string());
                    let access = NodeAccess::from_ssh_config(collator.ssh.as_ref());
                    let result = access.execute_shell(&run).await;
                    match result {
                        Ok(_) => {
                            let msg = format!(
                                "Command '{}' succeeded for {} on host {}.",
                                run,
                                collator.display_name(4),
                                collator.host(masked),
                            );
                            let _ = tx.send(Action::Input(InputAction::Success(msg)));
                        }
                        Err(e) => {
                            let msg = format!(
                                "Command '{}' failed for {} on host {}.",
                                run,
                                collator.display_name(4),
                                collator.host(masked),
                            );
                            let _ = tx.send(Action::Input(InputAction::Error(msg.clone())));
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "{}: {}",
                                msg, e
                            ))));
                        }
                    }
                });

                // Lock input so it can't be changed unless there's an error
                // and remove focus from the input field and start loading spinner
                let _ = self.tx.send(Action::Input(InputAction::Lock));
            }
            CommandKind::Uses(calls) => match calls {
                CustomCalls::RotateAndSetKeys => {
                    let runtime = collator.runtime();
                    let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                        return;
                    };
                    let api = chain.client().clone();
                    let stash = collator.key().stash();
                    let stash_identity = collator.display_name(3);

                    tokio::spawn(async move {
                        let (aura_key, proof) = match custom::rotate_keys(&collator).await {
                            Ok(result) => result,
                            Err(e) => {
                                let _ = tx.send(Action::Input(InputAction::Error(e.to_string())));
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to call rotate_keys: {}",
                                    e
                                ))));
                                return;
                            }
                        };
                        let call = Call::SetSessionKeys { aura_key, proof };
                        let Some((proxy_account_id, supported_proxy)) =
                            collator.get_proxy_for(runtime, &call)
                        else {
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "No signer is defined for {}",
                                runtime
                            ))));
                            return;
                        };
                        let proxy_identity =
                            to_compact_string(&proxy_account_id, runtime.account_format(), 6);

                        let at_block = match api.at_current_block().await.boxed() {
                            Ok(client) => client,
                            Err(e) => {
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to client at_current_block: {}",
                                    e
                                ))));
                                return;
                            }
                        };

                        let call_data_bytes = match runtime.build_call_data(
                            &at_block,
                            &stash,
                            call.clone(),
                            supported_proxy,
                        ) {
                            Ok(bytes) => bytes,
                            Err(e) => {
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to build call data: {}",
                                    e
                                ))));
                                return;
                            }
                        };

                        let qr_bytes = match build_transaction_qrcode(
                            &at_block,
                            &proxy_account_id,
                            &call_data_bytes,
                        )
                        .await
                        {
                            Ok(qr_bytes) => qr_bytes,
                            Err(e) => {
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to build QR data: {}",
                                    e
                                ))));
                                return;
                            }
                        };
                        info!("qr_bytes: 0x{}", hex::encode(&qr_bytes));

                        let spec_version = at_block.spec_version();
                        let ctx = Box::new(ConfirmationContext {
                            runtime,
                            spec_version,
                            proxy_identity,
                            proxy_account: proxy_account_id,
                            stash_identity,
                            call,
                            call_data_bytes,
                            qr_bytes,
                        });
                        let _ = tx.send(Action::Popup(PopupAction::ShowConfirmAndSign(ctx)));
                    });
                    // Lock input so it can't be changed unless there's an error
                    // and remove focus from the input field and start loading spinner
                    let _ = self.tx.send(Action::Input(InputAction::Lock));
                }
                CustomCalls::HasKeys => {
                    tokio::spawn(async move {
                        match custom::has_keys(&collator).await {
                            Ok(true) => {
                                let msg = format!(
                                    "Yes. Host {} contains the next session keys for {}.",
                                    collator.host(masked),
                                    collator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Success(msg)));
                            }
                            Ok(false) => {
                                let msg = format!(
                                    "No. Host {} does NOT have the next session keys for {}.",
                                    collator.host(masked),
                                    collator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Error(msg)));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Input(InputAction::Error(e.to_string())));
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to call has_keys: {}",
                                    e
                                ))));
                            }
                        }
                    });
                    let _ = self.tx.send(Action::Input(InputAction::Lock));
                }
                CustomCalls::HasQueuedKeys => {
                    tokio::spawn(async move {
                        match custom::has_queued_keys(&collator).await {
                            Ok(true) => {
                                let msg = format!(
                                    "Yes. Host {} contains the queued session keys for {}",
                                    collator.host(masked),
                                    collator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Success(msg)));
                            }
                            Ok(false) => {
                                let msg = format!(
                                    "No. Host {} does NOT have the queued session keys for {}.",
                                    collator.host(masked),
                                    collator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Error(msg)));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Input(InputAction::Error(e.to_string())));
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to call has_queued_keys: {}",
                                    e
                                ))));
                            }
                        }
                    });
                    let _ = self.tx.send(Action::Input(InputAction::Lock));
                }
            },
        }
    }

    pub fn handle_custom_command(&mut self, custom: CustomCommand, validator: Validator) {
        let tx = self.tx.clone();
        let masked = self.is_masked();

        match custom.kind {
            CommandKind::Shell { run, .. } => {
                tokio::spawn(async move {
                    let run = run.replace("{stash}", &validator.key().stash().to_string());
                    let access = NodeAccess::from_ssh_config(validator.ssh.as_ref());
                    let result = access.execute_shell(&run).await;
                    match result {
                        Ok(_) => {
                            let msg = format!(
                                "Command '{}' succeeded for {} on host {}.",
                                run,
                                validator.display_name(4),
                                validator.host(masked),
                            );
                            let _ = tx.send(Action::Input(InputAction::Success(msg)));
                        }
                        Err(e) => {
                            let msg = format!(
                                "Command '{}' failed for {} on host {}.",
                                run,
                                validator.display_name(4),
                                validator.host(masked),
                            );
                            let _ = tx.send(Action::Input(InputAction::Error(msg.clone())));
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "{}: {}",
                                msg, e
                            ))));
                        }
                    }
                });

                // Lock input so it can't be changed unless there's an error
                // and remove focus from the input field and start loading spinner
                let _ = self.tx.send(Action::Input(InputAction::Lock));
            }
            CommandKind::Uses(calls) => match calls {
                CustomCalls::RotateAndSetKeys => {
                    let runtime = validator
                        .runtime()
                        .asset_hub_runtime()
                        .expect("every relay has an AssetHub chain");

                    let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                        return;
                    };
                    let api = chain.client().clone();
                    let stash = validator.key().stash();
                    let stash_identity = validator.display_name(3);

                    tokio::spawn(async move {
                        let result = custom::rotate_keys(&validator).await;
                        match result {
                            Ok((keys, proof)) => {
                                let call = Call::SetKeys { keys, proof };
                                let Some((proxy_account_id, supported_proxy)) =
                                    validator.get_proxy_for(runtime, &call)
                                else {
                                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                                        "No signer is defined for {}",
                                        runtime
                                    ))));
                                    return;
                                };
                                let proxy_identity = to_compact_string(
                                    &proxy_account_id,
                                    runtime.account_format(),
                                    6,
                                );

                                let at_block = match api.at_current_block().await.boxed() {
                                    Ok(client) => client,
                                    Err(e) => {
                                        let _ = tx.send(Action::System(SystemAction::Error(
                                            format!("Failed to client at_current_block: {}", e),
                                        )));
                                        return;
                                    }
                                };

                                let call_data_bytes = match runtime.build_call_data(
                                    &at_block,
                                    &stash,
                                    call.clone(),
                                    supported_proxy,
                                ) {
                                    Ok(bytes) => bytes,
                                    Err(e) => {
                                        let _ = tx.send(Action::System(SystemAction::Error(
                                            format!("Failed to build call data: {}", e),
                                        )));
                                        return;
                                    }
                                };

                                let qr_bytes = match build_transaction_qrcode(
                                    &at_block,
                                    &proxy_account_id,
                                    &call_data_bytes,
                                )
                                .await
                                {
                                    Ok(qr_bytes) => qr_bytes,
                                    Err(e) => {
                                        let _ = tx.send(Action::System(SystemAction::Error(
                                            format!("Failed to build QR data: {}", e),
                                        )));
                                        return;
                                    }
                                };
                                info!("qr_bytes: 0x{}", hex::encode(&qr_bytes));

                                let spec_version = at_block.spec_version();
                                let ctx = Box::new(ConfirmationContext {
                                    runtime,
                                    spec_version,
                                    proxy_identity,
                                    proxy_account: proxy_account_id,
                                    stash_identity,
                                    call,
                                    call_data_bytes,
                                    qr_bytes,
                                });
                                let _ =
                                    tx.send(Action::Popup(PopupAction::ShowConfirmAndSign(ctx)));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Input(InputAction::Error(e.to_string())));
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to call rotate_keys: {}",
                                    e
                                ))));
                            }
                        }
                    });
                }
                CustomCalls::HasKeys => {
                    tokio::spawn(async move {
                        let result = custom::has_keys(&validator).await;
                        match result {
                            Ok(true) => {
                                let msg = format!(
                                    "Yes. Host {} contains the next session keys for {}.",
                                    validator.host(masked),
                                    validator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Success(msg)));
                            }
                            Ok(false) => {
                                let msg = format!(
                                    "No. Host {} does NOT have the next session keys for {}.",
                                    validator.host(masked),
                                    validator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Error(msg)));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Input(InputAction::Error(e.to_string())));
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to call has_keys: {}",
                                    e
                                ))));
                            }
                        }
                    });
                    // Lock input so it can't be changed unless there's an error
                    // and remove focus from the input field and start loading spinner
                    let _ = self.tx.send(Action::Input(InputAction::Lock));
                }
                CustomCalls::HasQueuedKeys => {
                    tokio::spawn(async move {
                        let result = custom::has_queued_keys(&validator).await;
                        match result {
                            Ok(true) => {
                                let msg = format!(
                                    "Yes. Host {} contains the queued session keys for {}",
                                    validator.host(masked),
                                    validator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Success(msg)));
                            }
                            Ok(false) => {
                                let msg = format!(
                                    "No. Host {} does NOT have the queued session keys for {}.",
                                    validator.host(masked),
                                    validator.display_name(4),
                                );
                                let _ = tx.send(Action::Input(InputAction::Error(msg)));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Input(InputAction::Error(e.to_string())));
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to call has_queued_keys: {}",
                                    e
                                ))));
                            }
                        }
                    });
                    // Lock input so it can't be changed unless there's an error
                    // and remove focus from the input field and start loading spinner
                    let _ = self.tx.send(Action::Input(InputAction::Lock));
                }
            },
        }
    }

    /// Handle decoded bytes from qr code when popup is in confirmation mode
    pub fn on_qr_decoded_signature(&mut self, signature_bytes: &[u8]) {
        if !self.popup.is_visible() {
            return;
        }

        if self.data_section() == Section::Validators {
            let Some(validator) = self.validators.get_selected() else {
                return;
            };

            if self.popup.is_confirmation_mode() {
                let runtime = validator
                    .runtime()
                    .asset_hub_runtime()
                    .expect("every relay has an AssetHub chain");
                let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                    return;
                };
                let Some(bytes) = self.popup.get_call_data_bytes() else {
                    return;
                };
                let api = chain.client().clone();
                let tx = self.tx.clone();

                if let Some(signer) = self.popup.get_proxy_account() {
                    sync::spawn_submit_call_data_with_signature(
                        &api,
                        runtime,
                        &signer,
                        &bytes,
                        signature_bytes,
                        &tx,
                    );
                }
            }
        };

        if self.data_section() == Section::Collators {
            let Some(collator) = self.collators.get_selected() else {
                return;
            };

            if self.popup.is_confirmation_mode() {
                let runtime = collator.runtime();
                let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                    return;
                };
                let Some(bytes) = self.popup.get_call_data_bytes() else {
                    return;
                };
                let api = chain.client().clone();
                let tx = self.tx.clone();

                if let Some(signer) = self.popup.get_proxy_account() {
                    sync::spawn_submit_call_data_with_signature(
                        &api,
                        runtime,
                        &signer,
                        &bytes,
                        signature_bytes,
                        &tx,
                    );
                }
            }
        };
    }

    /// Handle enter when a validator is selected and popup is in confirmation mode
    /// showing the call details and input field as password
    pub fn on_validator_confirm_enter(&mut self, validator: Validator) {
        let runtime = validator
            .runtime()
            .asset_hub_runtime()
            .expect("every relay has an AssetHub chain");

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };

        let Some(bytes) = self.popup.get_call_data_bytes() else {
            return;
        };
        let api = chain.client().clone();
        let tx = self.tx.clone();

        let result = self
            .popup
            .execute_with_password(|password| -> AppResult<()> {
                let password = Zeroizing::new(password.to_string());

                tokio::spawn(async move {
                    // Use spawn_blocking for CPU-intensive decrypt_json operation
                    let signer_result =
                        tokio::task::spawn_blocking(move || suno_signer::load_keypair(&password))
                            .await;

                    match signer_result {
                        Ok(Ok(signer)) => {
                            sync::spawn_sign_and_submit_call_data(
                                &api, runtime, &signer, &bytes, &tx,
                            );
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "Failed to load keypair: {}",
                                e
                            ))));
                            let _ = tx.send(Action::Input(InputAction::Error(
                                "Invalid password".to_string(),
                            )));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "Task failed: {}",
                                e
                            ))));
                            let _ = tx.send(Action::Input(InputAction::Error(
                                "Something went wrong, check errors and try again".to_string(),
                            )));
                        }
                    }
                });

                // Lock input so it can't be changed unless there's an error
                // and remove focus from the input field and start verification password spinner
                let _ = self.tx.send(Action::Input(InputAction::Lock));

                Ok(())
            });
        if let Err(e) = result {
            let _ = self
                .tx
                .send(Action::System(SystemAction::Error(e.to_string())));
            let _ = self.tx.send(Action::Input(InputAction::Error(
                "Something went wrong, check errors and try again".to_string(),
            )));
        }
    }

    pub fn on_collator_confirm_enter(&mut self, collator: Collator) {
        let runtime = collator.runtime();

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };

        let Some(bytes) = self.popup.get_call_data_bytes() else {
            return;
        };
        let api = chain.client().clone();
        let tx = self.tx.clone();

        let result = self
            .popup
            .execute_with_password(|password| -> AppResult<()> {
                let password = Zeroizing::new(password.to_string());

                tokio::spawn(async move {
                    // Use spawn_blocking for CPU-intensive decrypt_json operation
                    let signer_result =
                        tokio::task::spawn_blocking(move || suno_signer::load_keypair(&password))
                            .await;

                    match signer_result {
                        Ok(Ok(signer)) => {
                            sync::spawn_sign_and_submit_call_data(
                                &api, runtime, &signer, &bytes, &tx,
                            );
                        }
                        Ok(Err(e)) => {
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "Failed to load keypair: {}",
                                e
                            ))));
                            let _ = tx.send(Action::Input(InputAction::Error(
                                "Invalid password".to_string(),
                            )));
                        }
                        Err(e) => {
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "Task failed: {}",
                                e
                            ))));
                            let _ = tx.send(Action::Input(InputAction::Error(
                                "Something went wrong, check errors and try again".to_string(),
                            )));
                        }
                    }
                });

                // Lock input so it can't be changed unless there's an error
                // and remove focus from the input field and start verification password spinner
                let _ = self.tx.send(Action::Input(InputAction::Lock));

                Ok(())
            });
        if let Err(e) = result {
            let _ = self
                .tx
                .send(Action::System(SystemAction::Error(e.to_string())));
            let _ = self.tx.send(Action::Input(InputAction::Error(
                "Something went wrong, check errors and try again".to_string(),
            )));
        }
    }

    /// Handle input paste depending on the context
    pub fn on_input_paste(&mut self, data: String) {
        if !self.popup.is_visible() {
            return;
        }

        // The chain commands take an address, e.g. `add_validator <address>`
        let is_menu_section = matches!(self.data_section(), Section::Validators | Section::Chains);
        if is_menu_section && self.popup.get_mode() == PopupMode::Menu {
            self.popup.insert_input_paste_data(data);
        };
    }

    /// Cancel instruction.
    pub fn cancel(&mut self) {
        if !self.popup.is_visible() {
            return;
        }

        if self.popup.get_mode() == PopupMode::ThemeMenu {
            let catalog = CONFIG.theme_catalog();
            if let Some((name, _)) = catalog.get(self.theme_index_before_menu) {
                let name = name.clone();
                self.apply_theme_by_name(&name);
            }
        }

        // The popups opened from a command menu go back to it
        if matches!(
            self.popup.get_mode(),
            PopupMode::ScanAccount
                | PopupMode::Confirmation
                | PopupMode::ChainSpecs
                | PopupMode::Metadata
        ) && self.show_previous_popup()
        {
            self.focus = Focus::Popup;
            return;
        }

        if self.popup.can_close() {
            self.close_popup();
        }
    }

    /// Shows again the commands popup of the selected section, the one the current popup was
    /// opened from.
    fn show_previous_popup(&mut self) -> bool {
        match self.section {
            Section::Chains => self.show_chain_popup(),
            Section::Proxies => self.show_proxy_popup(),
            _ => match self.data_section() {
                Section::Validators => self.show_validator_popup(),
                Section::Collators => self.show_collator_popup(),
                _ => false,
            },
        }
    }

    /// Reset active selection
    pub fn reset_selection(&mut self) {
        // Esc removes the filter of the window displayed first
        if self.is_filtered() {
            self.edit_filter(|input| input.reset_as_filter());
            return;
        }

        self.chains.set_active(false);
        self.proxies.set_active(false);
        self.nominators.set_active(false);
        self.validators.set_focus(ListFocus::None);
        self.collators.set_focus(ListFocus::None);
        self.logs.clear_selection();
        if self.focus == Focus::Main {
            self.focus = Focus::None;
        }
    }

    /// Copy to clipboard
    pub fn copy_to_clipboard(&self) {
        if self.section.is_none() && self.window.is_logs_selected() && !self.popup.is_visible() {
            if let Some(text) = self.logs.selected_text() {
                self.set_clipboard_text(text);
            }
            return;
        }

        // The stash of the selected validator or collator
        if !self.popup.is_visible() {
            let address = match self.window {
                Window::Validators => self
                    .validators
                    .get_selected_ref()
                    .map(|v| v.to_address_string()),
                Window::Collators => self
                    .collators
                    .get_selected_ref()
                    .map(|c| c.to_address_string()),
                _ => None,
            };
            if let Some(address) = address {
                self.set_clipboard_text(address);
            }
            return;
        }

        if self.popup.is_confirmation_mode() {
            let Some(bytes) = self.popup.get_call_data_bytes() else {
                return;
            };
            self.set_clipboard_text(hex::encode(&bytes));
        }
    }

    fn set_clipboard_text(&self, text: String) {
        let mut clipboard = match Clipboard::new() {
            Ok(cb) => cb,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };

        if let Err(e) = clipboard.set_text(text) {
            error!("{}", e);
        }
    }
}
