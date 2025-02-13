use crate::actions::{
    Action, ChainAction, NavigationAction, PopupAction, StakingAction, SystemAction,
};
use crate::config::CONFIG;
use crate::errors::TuiError;
use crate::menu::Command;
use crate::section::Section;
use crate::widgets::{
    chains::ChainsListWidget, collators::CollatorsListWidget, validators::ValidatorsListWidget,
    validators_popup,
};
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use log::{info, warn};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, TuiError>;

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// The current selected section.
    pub section: Section,
    /// Holds the API clients for each supported runtime.
    pub chains: ChainsListWidget,
    /// Holds the validators list for the selected relay-chain.
    pub validators: ValidatorsListWidget,
    /// Holds the collators list for the selected relay-chain.
    pub collators: CollatorsListWidget,
    /// The sender to send actions to update the state to the app.
    pub tx: UnboundedSender<Action>,
    /// The receiver to handle actions sent from tx.
    pub rx: UnboundedReceiver<Action>,
    /// Is the popup menu open?
    pub is_popup_visible: bool,
}

impl App {
    /// Constructs a new instance of [`App`].
    pub fn new() -> Self {
        // Define the channel to send actions to update the app state.
        let (tx, rx) = unbounded_channel::<Action>();

        Self {
            running: true,
            section: Section::Chains,
            chains: ChainsListWidget::new(tx.clone()),
            validators: ValidatorsListWidget::default(),
            collators: CollatorsListWidget::default(),
            tx,
            rx,
            is_popup_visible: false,
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
        let events = EventHandler::new(1000);
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
            _ => Action::System(SystemAction::Noop),
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
                Action::Staking(act) => self.handle_staking_actions(act),
            }
        }
    }

    fn handle_system_actions(&mut self, action: SystemAction) {
        match action {
            SystemAction::Quit => self.quit(),
            SystemAction::Tick => self.tick(),
            SystemAction::Noop => self.noop(),
            SystemAction::Error(_err) => {
                // self.error(err)
            }
        }
    }

    fn handle_navigation_actions(&mut self, action: NavigationAction) {
        match action {
            NavigationAction::SectionUp => self.section_up(),
            NavigationAction::SectionDown => self.section_down(),
            NavigationAction::MoveUp => self.move_up(),
            NavigationAction::MoveDown => self.move_down(),
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
            ChainAction::Connection { runtime, state } => {
                self.chains.set_connection_state(runtime, state)
            }
        }
    }

    fn handle_staking_actions(&mut self, action: StakingAction) {
        match action {
            StakingAction::Chill => self.chill_attempt(),
            StakingAction::Bond => {}
            StakingAction::Unbond => {}
            StakingAction::ChangeRewardDestination => {}
            StakingAction::ChangeCommission => {}
            StakingAction::KickNominators => {}
            StakingAction::SetSessionKey => {}
        }
    }

    /// Handles the noop event of the terminal.
    pub fn error(&self, err: Box<dyn std::error::Error>) {
        warn!("TODO: HANDLE APPLICTaION ERRORS {}", err);
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
                if self.is_popup_visible {
                    self.validators.move_popup_up();
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
                if self.is_popup_visible {
                    self.validators.move_popup_down();
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
        if self.is_popup_visible {
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
        if self.is_popup_visible {
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

    /// Toggle menu popup status
    pub fn toggle_menu_popup(&mut self) {
        self.is_popup_visible = !self.is_popup_visible;
        match self.section {
            Section::Validators => {
                self.validators.init_popup_menu();
                self.validators.set_popup_visibility(self.is_popup_visible);
            }
            _ => {}
        };
    }

    /// Confirm and execute instruction.
    pub fn confirm(&mut self) {
        if !self.is_popup_visible {
            return;
        }
        match self.section {
            Section::Validators => match self.validators.popup.get_mode() {
                validators_popup::Mode::Menu => {
                    if let Some(entry) = self.validators.popup.get_selected() {
                        match entry.get_command() {
                            Command::Text(text) => match text.as_str() {
                                "cancel" => self.cancel(),
                                _ => {}
                            },
                            Command::Instruction(call) => match call {
                                validators_popup::Staking::Chill => self.chill_attempt(),
                                _ => {}
                            },
                        }
                    }
                }
                validators_popup::Mode::Confirm => {
                    if let Some(entry) = self.validators.popup.get_selected() {
                        match entry.get_command() {
                            Command::Text(text) => match text.as_str() {
                                "cancel" => self.cancel(),
                                _ => {}
                            },
                            Command::Instruction(call) => {
                                if let Some(validator) = self.validators.get_selected() {
                                    if let Some(chain_client) =
                                        self.chains.get_chain_client_by_runtime(validator.runtime())
                                    {
                                        match call {
                                            validators_popup::Staking::Chill => {
                                                info!("TODO: submit extrinsic!!");
                                                validator.chill();
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            _ => {}
        };
    }

    /// Cancel instruction.
    pub fn cancel(&mut self) {
        self.is_popup_visible = false;
    }

    /// Attempt chill instruction
    pub fn chill_attempt(&mut self) {
        self.is_popup_visible = true;
        match self.section {
            Section::Validators => {
                self.validators.init_popup_chill_attempt();
                self.validators.set_popup_visibility(self.is_popup_visible);
            }
            _ => {}
        };
    }
}
