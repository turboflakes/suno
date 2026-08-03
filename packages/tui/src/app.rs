use crate::bridge::{custom, sync, RuntimeCaller};
use crate::section::Section;
use crate::widgets::{
    chains::ChainsListWidget,
    collators::CollatorsListWidget,
    logs::LogsState,
    popup::{Mode as PopupMode, PopupWidget},
    validators::ValidatorsListWidget,
    window::Window,
};
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use arboard::Clipboard;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::{io, thread, time::Duration};
use suno_actions::network::ConnectionState;
use suno_actions::{
    Action, ChainAction, ConfirmationContext, InputAction, NavigationAction, PopupAction,
    ScannerAction, SystemAction, ThreadAction, TxAction, UpdateAction, ValidatorAction,
};
use suno_config::{CommandKind, CustomCalls, CustomCommand, NodeAccess, SupportedRuntime, CONFIG};
use suno_error::{Error, ResultExt};
use suno_primitives::entry::ToMethod;
use suno_primitives::{call::Call, display::to_compact_string, Validator};
use suno_qrcode::{scanner::Scanner, tx::build_qrcode};
use suno_tracing::LogEntry;
use suno_update::update;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use zeroize::Zeroizing;

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Error>;

// Constants
const TICK_RATE: u64 = 250;

/// Application active focus.
#[derive(Debug, Clone, Default)]
pub enum Focus {
    #[default]
    Main, // Arrows move the sections and tabs
    Input,   // Arrows move the cursor in the input/password fields
    Scanner, // Esc/Tab change mode state
    Popup,   // Esc/Tab change mode state
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
    /// Holds the API clients for each supported runtime.
    pub chains: ChainsListWidget,
    /// Holds the validators list for the selected relay-chain.
    pub validators: ValidatorsListWidget,
    /// Holds the collators list for the selected relay-chain.
    pub collators: CollatorsListWidget,
    /// The popup widget.
    pub popup: PopupWidget,
    /// Logs state.
    pub logs: LogsState,
    /// Is any sensitive data masked?
    pub masked: bool,
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

        Self {
            running: true,
            focus: Focus::default(),
            window: Window::default(),
            section: Section::default(),
            chains: ChainsListWidget::new(tx.clone()),
            validators: ValidatorsListWidget::new(),
            collators: CollatorsListWidget::default(),
            popup: PopupWidget::default(),
            logs: LogsState::new(rx_logs),
            masked: true,
            new_version: None,
            tx,
            rx,
        }
    }

    async fn init(&mut self) {
        self.check_for_update().await;
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

    fn is_masked(&self) -> bool {
        self.masked
    }

    pub async fn check_for_update(&mut self) {
        let new_version = suno_update::check_for_update().await.ok();
        self.new_version = new_version;
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
            NavigationAction::MoveUp => self.move_up(),
            NavigationAction::MoveDown => self.move_down(),
            NavigationAction::NextWindow => self.next_window(),
            NavigationAction::PrevWindow => self.prev_window(),
            NavigationAction::Reset => self.reset_selection(),
            NavigationAction::Copy => self.copy_to_clipboard(),
            NavigationAction::ToggleMask => self.toggle_mask(),
        }
    }

    fn handle_popup_actions(&mut self, action: PopupAction) {
        match action {
            PopupAction::Open => self.open_popup(),
            PopupAction::ConfirmAndSign(ctx) => {
                self.confirm_and_sign_popup(&ctx);
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
            InputAction::Char(new_char) => self.popup.insert_input_char(new_char),
            InputAction::Delete => self.popup.delete_input_char(),
            InputAction::CursorLeft => self.popup.move_cursor_left(),
            InputAction::CursorRight => self.popup.move_cursor_right(),
            InputAction::Enter => self.on_input_enter(),
            InputAction::Paste(data) => self.on_input_paste(data),
            InputAction::Error(msg) => {
                if (self.popup.is_menu_or_confirmation_mode()) && self.popup.invalidate_input(&msg)
                {
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

                match (is_updated, connection_state) {
                    (true, ConnectionState::Connected) => {
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
                                    sync::spawn_fetch_epoch_data(
                                        &api, block_hash, runtime, &self.tx,
                                    );

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

                                    sync::spawn_fetch_account_balance(
                                        &api,
                                        block_hash,
                                        runtime,
                                        &validator_keys,
                                        &self.tx,
                                    );

                                    if let Ok(proxy) = runtime.signer_account_id() {
                                        sync::spawn_fetch_validators_proxy_status(
                                            &api,
                                            block_hash,
                                            runtime,
                                            &validator_keys,
                                            &proxy,
                                            &self.tx,
                                        );
                                    };
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
                    (true, ConnectionState::BestBlockSubcriptionDropped(e)) => {
                        warn!("{}", e);
                        if let Some(chain) = self.chains.get_chain_by_runtime(chain_key) {
                            self.chains.subscribe_best_block(&chain);
                        }
                    }
                    (true, ConnectionState::FinalizedSubscriptionDropped(e)) => {
                        warn!("{}", e);
                        if let Some(chain) = self.chains.get_chain_by_runtime(chain_key) {
                            self.chains.subscribe_finalized_block(&chain);
                        }
                    }
                    (_, ConnectionState::Error(e)) => {
                        error!("{}", e);
                    }
                    _ => {}
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
                // And also a fetch to the account balance is being called here.
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
                    sync::spawn_fetch_account_balance(
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
                // NOTE: Fetch account balance after sub-chunk to keep balance in sync.
                let runtime = validator_key.runtime().asset_hub_runtime();
                if let Some((api, block_hash)) = self.chains.get_api_and_block_hash(runtime) {
                    let validator_keys = vec![validator_key];
                    sync::spawn_fetch_account_balance(
                        &api,
                        block_hash,
                        runtime,
                        &validator_keys,
                        &self.tx,
                    );
                }
            }
            ValidatorAction::AddProxy(validator_key, proxy) => {
                self.validators.add_proxy(&validator_key, proxy);
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
                info!("{message}");
                self.popup.update_transaction_status(&message);
                self.close_popup();
            }
            // NOTE: The following actions can just be logged.
            TxAction::InFinalizedBlock(block_hash) => {
                info!("Transaction finalized in block {block_hash}");
            }
            TxAction::Success => {
                info!("Transaction succeeded");
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
                let (ctrl_tx, mut ctrl_rx) = mpsc::unbounded_channel::<ThreadAction>();
                thread::spawn(move || {
                    let mut scanner = match Scanner::new() {
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
                self.on_qr_decoded_signature(&bytes);
            }
            ScannerAction::Frame(frame) => {
                self.popup.update_scanner_frame(frame);
            }
            ScannerAction::Error(e) => error!("{e}"),
        }
    }

    fn handle_update_actions(&mut self, action: UpdateAction) {
        match action {
            UpdateAction::Start => {
                self.popup.show_update_status();
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
                    .change_update_status(&format!("downloading {}", release.tag_name()));

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
                self.popup.change_update_status("validating");

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
    fn prev_window(&mut self) {
        self.window = self.window.prev();
    }

    /// Selects the next window.
    fn next_window(&mut self) {
        self.window = self.window.next();
    }

    /// Toggles the masked state of the application.
    fn toggle_mask(&mut self) {
        self.masked = !self.masked;
        self.validators.toggle_mask();
        self.popup.toggle_mask();
    }

    /// Open menu popup
    pub fn open_popup(&mut self) {
        if self.popup.is_visible() {
            return;
        }

        if self.section == Section::Validators {
            if !self.validators.is_proxy_valid() && !self.validators.is_commands_available() {
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

            self.popup.show_commands(active_era.index(), &validator);
            // Dispatch focus to the input field
            let _ = self.tx.send(Action::Input(InputAction::Editing));
        };
    }

    /// Open confirm and sign popup
    pub fn confirm_and_sign_popup(&mut self, ctx: &ConfirmationContext) {
        if !self.popup.is_visible() {
            return;
        }

        self.popup.show_confirm_and_sign(ctx);

        // If configured for QR signing, dispatch action to initialize scanner.
        // Otherwise, change focus to the input field for password signing.
        if ctx.runtime.is_qrcode_enabled() {
            let _ = self.tx.send(Action::Scanner(ScannerAction::Init));
        } else {
            let _ = self.tx.send(Action::Input(InputAction::Editing));
        }
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

        if self.section == Section::Validators {
            let Some(validator) = self.validators.get_selected() else {
                return;
            };

            match self.popup.get_mode() {
                PopupMode::Menu => {
                    self.on_menu_enter(validator);
                }
                PopupMode::Confirmation => {
                    self.on_confirm_enter(validator);
                }
                _ => {}
            }
        };
    }

    /// Handle enter when popup is in menu mode, showing available extrinsics/commands
    pub fn on_menu_enter(&mut self, validator: Validator) {
        let Some(call) = self.popup.get_input_parsed_call() else {
            return;
        };

        match call {
            Call::Custom(custom) => self.handle_custom_command(custom, validator),
            _ => self.handle_extrinsic_calls(call, validator),
        }
    }

    pub fn handle_extrinsic_calls(&mut self, call: Call, validator: Validator) {
        let runtime = validator.runtime().asset_hub_runtime();

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };
        let api = chain.client().clone();
        let tx = self.tx.clone();
        let stash = validator.key().stash();
        let proxy_account_id = match runtime.signer_account_id().boxed() {
            Ok(address) => address,
            Err(e) => {
                error!("{}", e);
                return;
            }
        };
        let supported_proxy = validator.get_proxy(runtime);
        let proxy_identity = to_compact_string(&proxy_account_id, runtime.account_format(), 6);
        let stash_identity = validator.display_name(3);

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

            let qr_bytes = match build_qrcode(&at_block, &proxy_account_id, &call_data_bytes).await
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

            let spec_version = at_block.spec_version();
            let ctx = ConfirmationContext {
                runtime,
                spec_version,
                proxy_identity,
                stash_identity,
                call,
                call_data_bytes,
                qr_bytes,
            };
            let _ = tx.send(Action::Popup(PopupAction::ConfirmAndSign(Box::new(ctx))));
        });
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
                    let runtime = validator.runtime().asset_hub_runtime();

                    let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                        return;
                    };
                    let api = chain.client().clone();
                    let stash = validator.key().stash();
                    let proxy_account_id = match runtime.signer_account_id().boxed() {
                        Ok(address) => address,
                        Err(e) => {
                            error!("{}", e);
                            return;
                        }
                    };
                    let stash_identity = validator.display_name(3);
                    let proxy_identity =
                        to_compact_string(&proxy_account_id, runtime.account_format(), 6);
                    let supported_proxy = validator.get_proxy(runtime);

                    tokio::spawn(async move {
                        let result = custom::rotate_keys(&validator).await;
                        match result {
                            Ok((keys, proof)) => {
                                let call = Call::SetKeys { keys, proof };

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

                                let qr_bytes = match build_qrcode(
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

                                let spec_version = at_block.spec_version();
                                let ctx = ConfirmationContext {
                                    runtime,
                                    spec_version,
                                    proxy_identity,
                                    stash_identity,
                                    call,
                                    call_data_bytes,
                                    qr_bytes,
                                };
                                let _ = tx.send(Action::Popup(PopupAction::ConfirmAndSign(
                                    Box::new(ctx),
                                )));
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

        if self.section == Section::Validators {
            let Some(validator) = self.validators.get_selected() else {
                return;
            };

            if self.popup.is_confirmation_mode() {
                let runtime = validator.runtime().asset_hub_runtime();
                let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
                    return;
                };
                let Some(entry) = self.popup.get_selected() else {
                    return;
                };

                let bytes = entry.as_bytes();
                let api = chain.client().clone();
                let tx = self.tx.clone();

                if let Ok(signer) = runtime.signer_account_id() {
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

    /// Handle enter when popup is in confirmation mode, showing call details and input field as password
    pub fn on_confirm_enter(&mut self, validator: Validator) {
        let runtime = validator.runtime().asset_hub_runtime();

        let Some(chain) = self.chains.get_chain_by_runtime(runtime) else {
            return;
        };

        let Some(entry) = self.popup.get_selected() else {
            return;
        };

        let bytes = entry.as_bytes();
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

        if self.section == Section::Validators && self.popup.get_mode() == PopupMode::Menu {
            self.popup.insert_input_paste_data(data);
        };
    }

    /// Cancel instruction.
    pub fn cancel(&mut self) {
        if !self.popup.is_visible() {
            return;
        }

        if self.popup.is_menu_or_confirmation_mode() {
            self.close_popup();
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

        if self.popup.is_confirmation_mode() {
            let Some(bytes_entry) = self.popup.get_selected() else {
                return;
            };
            let hex_bytes = bytes_entry.to_hex();

            let mut clipboard = match Clipboard::new() {
                Ok(cb) => cb,
                Err(e) => {
                    error!("{}", e);
                    return;
                }
            };

            if let Err(e) = clipboard.set_text(hex_bytes) {
                error!("{}", e);
            }
        }
    }
}
