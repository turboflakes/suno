use crate::bridge::sync;
use crate::error::TuiError;
use crate::menu::Command;
use crate::section::Section;
use crate::tab::Tab;
use crate::widgets::{
    chains::ChainsListWidget, collators::CollatorsListWidget, popup, popup::PopupWidget,
    validators::ValidatorsListWidget,
};
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use log::error;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use suno_actions::network::ConnectionState;
use suno_actions::{
    Action, ChainAction, NavigationAction, PopupAction, SystemAction, TxAction, ValidatorAction,
};
use suno_config::{SupportedRuntime, CONFIG};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, TuiError>;

// Constants
const TICK_RATE: u64 = 200;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// The current selected tab.
    pub tab: Tab,
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

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        // Define the channel to send actions to update the app state.
        let (tx, rx) = unbounded_channel::<Action>();

        Self {
            running: true,
            tab: Tab::Main,
            section: Section::Chains,
            chains: ChainsListWidget::new(tx.clone()),
            validators: ValidatorsListWidget::new(tx.clone()),
            collators: CollatorsListWidget::default(),
            popup: PopupWidget::default(),
            tx,
            rx,
            // is_popup_visible: false,
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
            Event::Key(key_event) => handle_key_events(key_event),
            Event::Mouse(_) => Action::System(SystemAction::Noop),
            Event::Resize(_, _) => Action::System(SystemAction::Noop),
            // _ => Action::System(SystemAction::Noop),
        };
        self.tx.send(action.clone())?;
        Ok(())
    }

    fn handle_actions(&mut self) {
        while let Ok(action) = self.rx.try_recv() {
            // Apply actionable messages to the application.
            match action {
                Action::System(act) => self.handle_system_actions(act),
                Action::Navigation(act) => self.handle_navigation_actions(act),
                Action::Popup(act) => self.handle_popup_actions(act),
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
        }
    }

    fn handle_popup_actions(&mut self, action: PopupAction) {
        match action {
            PopupAction::Toggle => self.toggle_menu_popup(),
            PopupAction::Confirm => self.confirm(),
            PopupAction::Cancel => self.cancel(),
        }
    }

    fn handle_chain_actions(&mut self, action: ChainAction) {
        match action {
            ChainAction::UpdateConnectionState(chain_key, connection_state) => {
                let is_updated = self
                    .chains
                    .update_connection_state(&chain_key, connection_state.clone());

                if is_updated && connection_state == ConnectionState::Connected {
                    let runtime = chain_key;
                    let validator_keys = self.validators.get_validator_keys_by_runtime(&runtime);
                    match runtime {
                        SupportedRuntime::Polkadot
                        | SupportedRuntime::Kusama
                        | SupportedRuntime::Paseo
                        | SupportedRuntime::Westend => {
                            if let Some((api, block_hash)) =
                                self.chains.get_api_and_block_hash(&runtime)
                            {
                                sync::spawn_fetch_epoch_data(&api, block_hash, &runtime, &self.tx);

                                sync::spawn_fetch_validators_authority_status(
                                    &api,
                                    block_hash,
                                    &runtime,
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
                                self.chains.get_api_and_block_hash(&runtime)
                            {
                                sync::spawn_fetch_era_data(&api, block_hash, &runtime, &self.tx);

                                sync::spawn_fetch_total_validators_count(
                                    &api, block_hash, &runtime, &self.tx,
                                );
                                sync::spawn_fetch_total_nominators_count(
                                    &api, block_hash, &runtime, &self.tx,
                                );

                                sync::spawn_fetch_validators_commission(
                                    &api,
                                    block_hash,
                                    &runtime,
                                    &validator_keys,
                                    &self.tx,
                                );

                                sync::spawn_fetch_validators_staking_ledger(
                                    &api,
                                    block_hash,
                                    &runtime,
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
                                self.chains.get_api_and_block_hash(&runtime)
                            {
                                sync::spawn_fetch_validators_identity(
                                    &api,
                                    block_hash,
                                    &runtime,
                                    &validator_keys,
                                    &self.tx,
                                )
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
                        if let Some(chain) = self.chains.get_chain_by_runtime(&runtime) {
                            let api = chain.client();
                            let validator_keys =
                                self.validators.get_validator_keys_by_runtime(&runtime);

                            sync::spawn_fetch_validators_points(
                                api,
                                block_hash,
                                &runtime,
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
                        if let Some((api, block_hash)) =
                            self.chains.get_api_and_block_hash(&runtime)
                        {
                            sync::spawn_fetch_active_validators_count(
                                &api,
                                block_hash,
                                &runtime,
                                era.index(),
                                &self.tx,
                            );

                            sync::spawn_fetch_active_nominators_count(
                                &api,
                                block_hash,
                                &runtime,
                                era.index(),
                                &self.tx,
                            );

                            sync::spawn_fetch_total_staked(
                                &api,
                                block_hash,
                                &runtime,
                                era.index(),
                                &self.tx,
                            );

                            let validator_keys =
                                self.validators.get_validator_keys_by_runtime(&runtime);

                            sync::spawn_fetch_validators_era_points(
                                &api,
                                block_hash,
                                &runtime,
                                era.index(),
                                &validator_keys,
                                &self.tx,
                            );

                            sync::spawn_fetch_validators_stake_overview(
                                &api,
                                block_hash,
                                &runtime,
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
            ValidatorAction::SubmitChill => self.chill_attempt(),
            ValidatorAction::SubmitBond => {}
            ValidatorAction::SubmitUnbond => {}
            ValidatorAction::SubmitChangeRewardDestination => {}
            ValidatorAction::SubmitChangeCommission => {}
            ValidatorAction::SubmitKickNominators => {}
            ValidatorAction::SubmitSetSessionKey => {}
            ValidatorAction::UpdateCommission(validator_key, commission) => {
                self.validators
                    .update_commission(&validator_key, commission);
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
            ValidatorAction::UpdateStatus(validator_key, status) => {
                self.validators.update_status(&validator_key, status);
            }
        }
    }

    fn handle_transaction_actions(&mut self, action: TxAction) {
        match action {
            TxAction::Processing => {
                self.popup.show_transaction_status();
            }
            TxAction::Sent => {
                self.popup.update_transaction_status("transaction sent");
            }
            TxAction::InBestBlock(block_hash) => {
                let message = format!("transaction in block {block_hash}");
                self.popup.update_transaction_status(&message);
            }
            TxAction::InFinalizedBlock(block_hash) => {
                let message = format!("finalized in block {block_hash}");
                self.popup.update_transaction_status(&message);
            }
            TxAction::Success => {
                self.popup.hide();
            }
            TxAction::Error(err) => {}
        }
    }

    /// Handles application errors.
    pub fn error(&self, err: Box<dyn std::error::Error>) {
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
        if self.popup.is_visible() {
            return;
        }
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
        if self.popup.is_visible() {
            return;
        }
        let config = CONFIG.clone();
        self.section = self.section.down(&config.features);
        self.chains.set_active(self.section == Section::Chains);
        self.validators
            .set_active(self.section == Section::Validators);
        self.collators
            .set_active(self.section == Section::Collators);
    }

    /// Selects the previous tab.
    fn prev_tab(&mut self) {
        self.tab = self.tab.prev();
    }

    /// Selects the next tab.
    fn next_tab(&mut self) {
        self.tab = self.tab.next();
    }

    /// Toggle menu popup status
    pub fn toggle_menu_popup(&mut self) {
        match self.section {
            Section::Validators => {
                if self.popup.is_visible() {
                    self.popup.hide();
                } else {
                    self.popup.show_menu();
                }
            }
            _ => {}
        };
    }

    /// Confirm and execute instruction.
    pub fn confirm(&mut self) {
        if !self.popup.is_visible() {
            return;
        }
        match self.section {
            Section::Validators => match self.popup.get_mode() {
                popup::Mode::Menu => {
                    if let Some(entry) = self.popup.get_selected() {
                        match entry.get_command() {
                            Command::Text(text) => match text.as_str() {
                                "cancel" => self.cancel(),
                                _ => {}
                            },
                            Command::Instruction(call) => match call {
                                popup::Staking::Chill => self.chill_attempt(),
                                _ => {}
                            },
                        }
                    }
                }
                popup::Mode::Confirm => {
                    if let Some(entry) = self.popup.get_selected() {
                        match entry.get_command() {
                            Command::Text(text) => match text.as_str() {
                                "cancel" => self.cancel(),
                                _ => {}
                            },
                            Command::Instruction(call) => {
                                if let Some(validator) = self.validators.get_selected() {
                                    match call {
                                        popup::Staking::Chill => {
                                            let runtime = validator.runtime().asset_hub_runtime();
                                            if let Some(chain) =
                                                self.chains.get_chain_by_runtime(&runtime)
                                            {
                                                let api = chain.client();
                                                let account_key = validator.key();
                                                // TODO: Implement spawn_call_chill
                                                sync::spawn_call_remark(
                                                    &api,
                                                    &runtime,
                                                    &account_key,
                                                    &self.tx,
                                                );
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                    }
                }
                _ => {}
            },
            _ => {}
        };
    }

    /// Cancel instruction.
    pub fn cancel(&mut self) {
        self.popup.hide();
    }

    /// Try chill instruction
    pub fn chill_attempt(&mut self) {
        match self.section {
            Section::Validators => {
                self.popup.confirm_chill_attempt();
            }
            _ => {}
        };
    }
}
