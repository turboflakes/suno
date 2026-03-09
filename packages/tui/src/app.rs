use crate::bridge::{sync, RuntimeCaller};
use crate::error::TuiError;
use crate::section::Section;
use crate::widgets::{
    chains::ChainsListWidget,
    collators::CollatorsListWidget,
    popup::{Mode as PopupMode, PopupWidget},
    validators::ValidatorsListWidget,
};
use crate::window::Window;
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use arboard::Clipboard;
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use suno_actions::network::ConnectionState;
use suno_actions::{
    Action, ChainAction, InputAction, NavigationAction, PopupAction, SystemAction, TxAction,
    ValidatorAction,
};
use suno_config::{SupportedRuntime, CONFIG};
use suno_error::{Error, ResultExt};
use suno_primitives::{call::Call, display::to_compact_string};
use suno_signer::get_address_from_json_file;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, TuiError>;

// Constants
const TICK_RATE: u64 = 250;

/// Application active focus.
#[derive(Debug, Clone, Default)]
pub enum Focus {
    #[default]
    Main, // Arrows move the sections and tabs
    Input, // Arrows move the cursor in the input/password fields
    Popup, // Esc/Tab change mode state
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Application active focus, useful to determine how to handle keyboard events.
    pub focus: Focus,
    /// The current selected window.
    pub window: Window,
    /// The current selected section.
    pub section: Section,
    /// Holds the API clients for each supported runtime.
    pub chains: ChainsListWidget,
    /// Holds the validators list for the selected relay-chain.
    pub validators: ValidatorsListWidget,
    /// Holds the collators list for the selected relay-chain.
    pub collators: CollatorsListWidget,
    /// The popup widget.
    pub popup: PopupWidget,
    /// The sender to send actions to update the state to the app.
    pub tx: UnboundedSender<Action>,
    /// The receiver to handle actions sent from tx.
    pub rx: UnboundedReceiver<Action>,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        // Define the channel to send actions to update the app state.
        let (tx, rx) = unbounded_channel::<Action>();

        Self {
            running: true,
            focus: Focus::default(),
            window: Window::default(),
            section: Section::default(),
            chains: ChainsListWidget::new(tx.clone()),
            validators: ValidatorsListWidget::new(),
            collators: CollatorsListWidget::default(),
            popup: PopupWidget::default(),
            tx,
            rx,
        }
    }

    async fn init(&mut self) {
        self.chains.on_init().await;
        self.validators.on_init();
        self.collators.on_init();
        // if let Some(chain) = self.chains.get_selected() {
        //     let tx = self.tx.clone();
        //     self.validators.on_chain_selected(chain.clone(), tx);
        //     let tx = self.tx.clone();
        //     self.collators.on_chain_selected(chain.clone(), tx);
        // }
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
                Action::Chain(act) => self.handle_chain_actions(act),
                Action::Validator(act) => self.handle_validator_actions(act),
                Action::Transaction(act) => self.handle_transaction_actions(act),
            }
        }
    }

    fn handle_system_actions(&mut self, action: SystemAction) {
        match action {
            SystemAction::Quit => self.quit(),
            SystemAction::Tick => self.tick(),
            SystemAction::Noop => self.noop(),
            SystemAction::Error(err) => error!("{err}"),
        }
    }

    fn handle_navigation_actions(&mut self, action: NavigationAction) {
        match action {
            NavigationAction::SectionUp => self.section_up(),
            NavigationAction::SectionDown => self.section_down(),
            NavigationAction::MoveUp => self.move_up(),
            NavigationAction::MoveDown => self.move_down(),
            NavigationAction::NextTab => self.next_tab(),
            NavigationAction::PrevTab => self.prev_tab(),
            NavigationAction::Reset => self.reset_selection(),
            NavigationAction::Copy => self.copy_to_clipboard(),
        }
    }

    fn handle_popup_actions(&mut self, action: PopupAction) {
        match action {
            PopupAction::Open => self.open_popup(),
            PopupAction::ConfirmAndSign(
                runtime,
                spec_version,
                proxy_identity,
                stash_identity,
                call,
                bytes,
            ) => {
                self.confirm_and_sign_popup(
                    runtime,
                    spec_version,
                    proxy_identity,
                    stash_identity,
                    *call,
                    bytes,
                );
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
            InputAction::Editing => {
                if self.popup.set_input_focus() {
                    self.focus = Focus::Input;
                }
            }
            InputAction::Unfocus => {
                self.popup.clear_input_focus();
                self.focus = Focus::Popup;
            }
            InputAction::Lock => {
                self.popup.set_lock_mode();
                self.focus = Focus::Popup;
            }
            InputAction::AutoComplete => {
                self.popup.set_input_autocomplete();
            }
            InputAction::Char(new_char) => self.popup.insert_input_char(new_char),
            InputAction::Delete => self.popup.delete_input_char(),
            InputAction::CursorLeft => self.popup.move_cursor_left(),
            InputAction::CursorRight => self.popup.move_cursor_right(),
            InputAction::Enter => self.handle_input_enter(),
            InputAction::Paste(data) => self.handle_input_paste(data),
            InputAction::Error(msg) => {
                if self.popup.get_mode() == PopupMode::Locked && self.popup.invalidate_input(&msg) {
                    self.popup.set_confirm_mode();
                    self.focus = Focus::Input;
                }
            }
        }
    }

    fn handle_chain_actions(&mut self, action: ChainAction) {
        match action {
            ChainAction::UpdateConnectionState(chain_key, connection_state) => {
                let is_updated = self
                    .chains
                    .update_connection_state(&chain_key, connection_state.clone());

                let proxy = match get_address_from_json_file() {
                    Ok(address) => address,
                    Err(e) => {
                        let _ = self
                            .tx
                            .send(Action::System(SystemAction::Error(e.to_string())));
                        return;
                    }
                };

                if is_updated && connection_state == ConnectionState::Connected {
                    let runtime = chain_key;
                    let validator_keys = self.validators.get_validator_keys_by_runtime(runtime);
                    match runtime {
                        SupportedRuntime::Polkadot
                        | SupportedRuntime::Kusama
                        | SupportedRuntime::Paseo
                        | SupportedRuntime::Westend => {
                            if let Some((api, block_hash)) =
                                self.chains.get_api_and_block_hash(runtime)
                            {
                                sync::spawn_fetch_epoch_data(&api, block_hash, runtime, &self.tx);

                                sync::spawn_fetch_validators_authority_status(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );

                                sync::spawn_fetch_validators_queued_keys(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );

                                sync::spawn_fetch_validators_next_keys(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );

                                sync::spawn_fetch_validators_proxy_status(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &proxy,
                                    &self.tx,
                                );
                            }
                        }
                        SupportedRuntime::AssetHubPolkadot
                        | SupportedRuntime::AssetHubKusama
                        | SupportedRuntime::AssetHubPaseo
                        | SupportedRuntime::AssetHubWestend => {
                            if let Some((api, block_hash)) =
                                self.chains.get_api_and_block_hash(runtime)
                            {
                                sync::spawn_fetch_era_data(&api, block_hash, runtime, &self.tx);

                                sync::spawn_fetch_total_validators_count(
                                    &api, block_hash, runtime, &self.tx,
                                );
                                sync::spawn_fetch_total_nominators_count(
                                    &api, block_hash, runtime, &self.tx,
                                );

                                sync::spawn_fetch_validators_prefs_next(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );

                                sync::spawn_fetch_validators_staking_ledger(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );

                                sync::spawn_fetch_validators_payee(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );

                                sync::spawn_fetch_validators_proxy_status(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &proxy,
                                    &self.tx,
                                );

                                sync::spawn_fetch_account_balance(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );
                            }
                        }
                        SupportedRuntime::PeoplePolkadot
                        | SupportedRuntime::PeopleKusama
                        | SupportedRuntime::PeoplePaseo
                        | SupportedRuntime::PeopleWestend => {
                            if let Some((api, block_hash)) =
                                self.chains.get_api_and_block_hash(runtime)
                            {
                                sync::spawn_fetch_validators_identity(
                                    &api,
                                    block_hash,
                                    runtime,
                                    &validator_keys,
                                    &self.tx,
                                );
                            }
                        }
                        _ => {}
                    }
                }
            }
            ChainAction::UpdateBestBlock(chain_key, block_number) => {
                self.chains.update_best_block(&chain_key, block_number);
            }
            ChainAction::UpdateFinalizedBlock(chain_key, block_number, block_hash) => {
                self.chains
                    .update_finalized_block(&chain_key, block_number, block_hash);

                // Fetch data relevant to be synced at every finalized block, eg. all validators points
                let runtime = chain_key;
                match runtime {
                    SupportedRuntime::Polkadot
                    | SupportedRuntime::Kusama
                    | SupportedRuntime::Paseo
                    | SupportedRuntime::Westend => {
                        if let Some(chain) = self.chains.get_chain_by_runtime(runtime) {
                            let api = chain.client();
                            let validator_keys =
                                self.validators.get_validator_keys_by_runtime(runtime);

                            sync::spawn_fetch_validators_points(
                                api,
                                block_hash,
                                runtime,
                                &validator_keys,
                                &self.tx,
                            )
                        }
                    }
                    _ => {}
                }
            }
            ChainAction::UpdateEra(chain_key, era) => {
                self.chains.update_era(&chain_key, era.clone());

                // Fetch data relevant to be synced whenever era changes
                let runtime = chain_key;
                match runtime {
                    SupportedRuntime::AssetHubPolkadot
                    | SupportedRuntime::AssetHubKusama
                    | SupportedRuntime::AssetHubPaseo
                    | SupportedRuntime::AssetHubWestend => {
                        if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime)
                        {
                            sync::spawn_fetch_active_validators_count(
                                &api,
                                block_hash,
                                runtime,
                                era.index(),
                                &self.tx,
                            );

                            sync::spawn_fetch_active_nominators_count(
                                &api,
                                block_hash,
                                runtime,
                                era.index(),
                                &self.tx,
                            );

                            sync::spawn_fetch_total_staked(
                                &api,
                                block_hash,
                                runtime,
                                era.index(),
                                &self.tx,
                            );

                            let validator_keys =
                                self.validators.get_validator_keys_by_runtime(runtime);

                            sync::spawn_fetch_validators_prefs(
                                &api,
                                block_hash,
                                runtime,
                                era.index(),
                                &validator_keys,
                                &self.tx,
                            );

                            sync::spawn_fetch_validators_era_points(
                                &api,
                                block_hash,
                                runtime,
                                era.index(),
                                &validator_keys,
                                &self.tx,
                            );

                            sync::spawn_fetch_validators_stake_overview(
                                &api,
                                block_hash,
                                runtime,
                                era.index(),
                                &validator_keys,
                                &self.tx,
                            );
                        }
                    }
                    _ => {}
                }
            }
            ChainAction::UpdateEpoch(chain_key, epoch) => {
                self.chains.update_epoch(&chain_key, epoch);

                // Fetch data relevant to be synced whenever session changes
                //
                let runtime = chain_key;
                let validator_keys = self.validators.get_validator_keys_by_runtime(runtime);
                match runtime {
                    SupportedRuntime::Polkadot
                    | SupportedRuntime::Kusama
                    | SupportedRuntime::Paseo
                    | SupportedRuntime::Westend => {
                        if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime)
                        {
                            sync::spawn_fetch_validators_authority_status(
                                &api,
                                block_hash,
                                runtime,
                                &validator_keys,
                                &self.tx,
                            );

                            sync::spawn_fetch_validators_queued_keys(
                                &api,
                                block_hash,
                                runtime,
                                &validator_keys,
                                &self.tx,
                            );

                            sync::spawn_fetch_validators_next_keys(
                                &api,
                                block_hash,
                                runtime,
                                &validator_keys,
                                &self.tx,
                            );
                        }
                    }
                    SupportedRuntime::AssetHubPolkadot
                    | SupportedRuntime::AssetHubKusama
                    | SupportedRuntime::AssetHubPaseo
                    | SupportedRuntime::AssetHubWestend => {
                        if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime)
                        {
                            sync::spawn_fetch_total_validators_count(
                                &api, block_hash, runtime, &self.tx,
                            );
                            sync::spawn_fetch_total_nominators_count(
                                &api, block_hash, runtime, &self.tx,
                            );
                        }
                    }
                    _ => {}
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

            _ => {}
        }
    }

    fn handle_validator_actions(&mut self, action: ValidatorAction) {
        match action {
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
                let runtime = validator_key.runtime().asset_hub_runtime();
                if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) {
                    let validator_keys = vec![validator_key];
                    sync::spawn_fetch_validators_staking_ledger(
                        &api,
                        block_hash,
                        runtime,
                        &validator_keys,
                        &self.tx,
                    );
                }
            }
            ValidatorAction::SubChunkFromStakeLedger(validator_key, chunk) => {
                self.validators
                    .sub_chunk_from_stake_ledger(&validator_key, chunk);
            }
            ValidatorAction::UpdateProxyStatus(validator_key, is_valid) => {
                self.validators
                    .update_proxy_status(&validator_key, is_valid);
            }
            ValidatorAction::UpdateBalance(validator_key, balance) => {
                self.validators.update_balance(&validator_key, balance);
            }
        }
    }

    fn handle_transaction_actions(&mut self, action: TxAction) {
        match action {
            TxAction::Processing => {
                self.popup.show_transaction_status();
                // Switch app focus to main while rendering transaction status
                self.focus = Focus::Main;
            }
            TxAction::Message(message) => {
                self.popup.update_transaction_status(message);
            }
            TxAction::InBestBlock(block_hash) => {
                let message = format!("transaction in block {block_hash}");
                self.popup.update_transaction_status(&message);
            }
            TxAction::InFinalizedBlock(block_hash) => {
                let message = format!("transaction finalized in block {block_hash}");
                self.popup.update_transaction_status(&message);
            }
            TxAction::Success => {
                self.close_popup();
            }
            TxAction::Error(err) => {
                error!("Transaction error: {}", err);
                self.close_popup();
            }
        }
    }

    /// Handles application errors.
    pub fn error(&self, err: Error) {
        error!("{}", err);
    }

    /// Handles the noop event of the terminal.
    pub fn noop(&self) {}

    /// Handles the tick event of the terminal.
    pub fn tick(&self) {}

    /// Set running to false to quit the application.
    pub fn quit(&mut self) {
        self.running = false;
    }

    /// Moves row selection up.
    pub fn move_up(&mut self) {
        match self.section {
            Section::Chains => {
                self.chains.move_up();
            }
            Section::Validators => {
                if self.popup.is_visible() {
                    self.popup.move_up();
                } else {
                    self.validators.move_up();
                }
            }
            Section::Collators => {
                self.collators.move_up();
            }
            _ => {}
        };
    }

    /// Moves row selection down.
    pub fn move_down(&mut self) {
        match self.section {
            Section::Chains => {
                self.chains.move_down();
            }
            Section::Validators => {
                if self.popup.is_visible() {
                    self.popup.move_down();
                } else {
                    self.validators.move_down();
                }
            }
            Section::Collators => {
                self.collators.move_down();
            }
            _ => {}
        };
    }

    /// Moves the active section up.
    pub fn section_up(&mut self) {
        let config = CONFIG.clone();
        self.section = self.section.up(&config.features);
        self.chains.set_active(self.section == Section::Chains);
        self.validators
            .set_active(self.section == Section::Validators);
        self.collators
            .set_active(self.section == Section::Collators);
    }

    /// Moves the active section down.
    pub fn section_down(&mut self) {
        let config = CONFIG.clone();
        self.section = self.section.down(&config.features);
        self.chains.set_active(self.section == Section::Chains);
        self.validators
            .set_active(self.section == Section::Validators);
        self.collators
            .set_active(self.section == Section::Collators);
    }

    /// Selects the previous window.
    fn prev_tab(&mut self) {
        self.window = self.window.prev();
    }

    /// Selects the next tab.
    fn next_tab(&mut self) {
        self.window = self.window.next();
    }

    /// Open menu popup
    pub fn open_popup(&mut self) {
        if self.popup.is_visible() {
            return;
        }

        if self.section == Section::Validators {
            if !self.validators.is_proxy_valid() {
                return;
            }

            let Some(validator) = self.validators.get_selected() else {
                return;
            };

            let Some(chain) = self
                .chains
                .get_chain_by_runtime(validator.runtime().asset_hub_runtime())
            else {
                return;
            };

            let Some(active_era) = chain.era() else {
                return;
            };

            self.popup.show_extrinsics(active_era.index(), validator);
            // Dispatch focus to the input field
            let _ = self.tx.send(Action::Input(InputAction::Editing));
        };
    }

    /// Open confirm and sign popup
    pub fn confirm_and_sign_popup(
        &mut self,
        runtime: SupportedRuntime,
        spec_version: u32,
        proxy_identity: String,
        stash_identity: String,
        call: Call,
        bytes: Vec<u8>,
    ) {
        if !self.popup.is_visible() {
            return;
        }

        self.popup.init_confirm_and_sign(
            runtime,
            spec_version,
            proxy_identity,
            stash_identity,
            call,
            bytes,
        );
        // Dispatch focus to the input field
        let _ = self.tx.send(Action::Input(InputAction::Editing));
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
    pub fn handle_input_enter(&mut self) {
        if !self.popup.is_visible() {
            return;
        }

        if self.section == Section::Validators {
            let Some(validator) = self.validators.get_selected() else {
                return;
            };

            match self.popup.get_mode() {
                PopupMode::Menu => {
                    let Some(call) = self.popup.get_input_parsed_call() else {
                        return;
                    };
                    // NOTE: Specific case where some calls are meant to be sent to RC and not AH
                    let runtime = if matches!(call, Call::SetSessionKeys { .. }) {
                        validator.runtime().relay_chain()
                    } else {
                        validator.runtime().asset_hub_runtime()
                    };

                    let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                        return;
                    };
                    let api = chain.client().clone();
                    let tx = self.tx.clone();
                    let stash = validator.key().stash();
                    let stash_identity = validator.display_name(3);
                    let proxy_identity = match get_address_from_json_file() {
                        Ok(address) => to_compact_string(&address, runtime.account_format(), 6),
                        Err(e) => {
                            self.error(e.into());
                            return;
                        }
                    };

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
                        let spec_version = at_block.spec_version();

                        match runtime.build_call_data(&at_block, &stash, call.clone()) {
                            Ok(bytes) => {
                                let _ = tx.send(Action::Popup(PopupAction::ConfirmAndSign(
                                    runtime,
                                    spec_version,
                                    proxy_identity,
                                    stash_identity,
                                    Box::new(call),
                                    bytes,
                                )));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::System(SystemAction::Error(format!(
                                    "Failed to build_call_data: {}",
                                    e
                                ))));
                            }
                        }
                    });
                }
                PopupMode::Confirm => {
                    let Some(call) = self.popup.get_selected_call() else {
                        return;
                    };
                    // NOTE: Specific case where some calls are meant to be sent to RC and not AH
                    let runtime = if matches!(call, Call::SetSessionKeys { .. }) {
                        validator.runtime().relay_chain()
                    } else {
                        validator.runtime().asset_hub_runtime()
                    };

                    let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                        return;
                    };

                    let Some(entry) = self.popup.get_selected() else {
                        return;
                    };

                    let bytes = entry.as_bytes();
                    let api = chain.client().clone();
                    let tx = self.tx.clone();

                    let result =
                        self.popup
                            .execute_with_password(|password| -> Result<(), TuiError> {
                                let password = password.to_string();

                                tokio::spawn(async move {
                                    // Use spawn_blocking for CPU-intensive decrypt_json operation
                                    let signer_result = tokio::task::spawn_blocking(move || {
                                        suno_signer::load_keypair(&password)
                                    })
                                    .await;

                                    match signer_result {
                                        Ok(Ok(signer)) => {
                                            sync::spawn_sign_and_submit(
                                                &api, runtime, &signer, &bytes, &tx,
                                            );
                                        }
                                        Ok(Err(e)) => {
                                            let _ = tx.send(Action::System(SystemAction::Error(
                                                format!("Failed to load keypair: {}", e),
                                            )));
                                            let _ = tx.send(Action::Input(InputAction::Error(
                                                "Invalid password".to_string(),
                                            )));
                                        }
                                        Err(e) => {
                                            let _ = tx.send(Action::System(SystemAction::Error(
                                                format!("Task failed: {}", e),
                                            )));
                                            let _ = tx.send(Action::Input(InputAction::Error(
                                                "Something went wrong, check errors and try again"
                                                    .to_string(),
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
                _ => {}
            }
        };
    }

    // Handle input paste depending on the context
    pub fn handle_input_paste(&mut self, data: String) {
        if !self.popup.is_visible() {
            return;
        }

        if self.section == Section::Validators && self.popup.get_mode() == PopupMode::Menu {
            self.popup.insert_input_paste_data(data);
        };
    }

    /// Cancel instruction.
    pub fn cancel(&mut self) {
        match self.popup.get_mode() {
            PopupMode::Menu | PopupMode::Confirm => {
                self.close_popup();
            }
            _ => {}
        }
    }

    /// Reset active selection
    pub fn reset_selection(&mut self) {
        self.chains.set_active(false);
        self.validators.set_active(false);
        self.collators.set_active(false);
    }

    /// Copy to clipboard
    pub fn copy_to_clipboard(&self) {
        if !self.popup.is_visible() {
            return;
        }

        match self.popup.get_mode() {
            PopupMode::Confirm | PopupMode::Locked => {
                let Some(bytes_entry) = self.popup.get_selected() else {
                    return;
                };
                let hex_bytes = bytes_entry.to_hex();

                let mut clipboard = match Clipboard::new() {
                    Ok(cb) => cb,
                    Err(e) => {
                        self.error(e.into());
                        return;
                    }
                };

                if let Err(e) = clipboard.set_text(hex_bytes) {
                    self.error(e.into());
                }
            }
            _ => {}
        }
    }
}
