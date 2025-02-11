use crate::config::{Features, SupportedRuntime, CONFIG};
use crate::section::Section;
use crate::widgets::{
    chains::{ChainsListWidget, ConnectionState},
    collators::CollatorsListWidget,
    validators::ValidatorsListWidget,
};
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use log::info;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

/// Application result type.
pub type AppResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Application actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Quit,
    Tick,
    SectionUp,
    SectionDown,
    MoveUp,
    MoveDown,
    TogglePopup,
    ChainConnection(SupportedRuntime, ConnectionState),
    Chill,
    Bond,
    Unbond,
    ChangeRewardDestination,
    ChangeCommission,
    KickNominators,
    SetSessionKey,
    Noop,
}

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
        let (tx, rx) = unbounded_channel();

        Self {
            running: true,
            section: Section::Chains,
            chains: ChainsListWidget::default(),
            validators: ValidatorsListWidget::default(),
            collators: CollatorsListWidget::default(),
            tx,
            rx,
            is_popup_visible: false,
        }
    }

    async fn init(&mut self) {
        self.chains.on_init(&self.tx).await;
        self.validators.on_init(&self.tx);
        self.collators.on_init(&self.tx);
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
            // Update the application.
            self.update();
        }

        // Exit the user interface.
        tui.exit()?;
        Ok(())
    }

    fn handle_events(&mut self, event: Event) -> AppResult<()> {
        let action = match event {
            Event::Tick => Action::Tick,
            Event::Key(key_event) => handle_key_events(key_event),
            Event::Mouse(_) => Action::Noop,
            Event::Resize(_, _) => Action::Noop,
            _ => Action::Noop,
        };
        self.tx.send(action.clone())?;
        Ok(())
    }

    fn update(&mut self) {
        while let Ok(action) = self.rx.try_recv() {
            // Apply actionable messages to the application.
            match action {
                Action::Quit => self.quit(),
                Action::Tick => self.tick(),
                Action::SectionUp => self.section_up(),
                Action::SectionDown => self.section_down(),
                Action::MoveUp => self.move_up(),
                Action::MoveDown => self.move_down(),
                Action::TogglePopup => self.toggle_menu_popup(),
                Action::ChainConnection(runtime, connection) => {
                    self.chains.set_connection_state(runtime, connection)
                }
                Action::Chill => self.chill_attempt(),
                Action::Bond => {}
                Action::Unbond => {}
                Action::ChangeRewardDestination => {}
                Action::ChangeCommission => {}
                Action::KickNominators => {}
                Action::SetSessionKey => {}
                Action::Noop => self.noop(),
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
                self.validators.set_popup_visibility(self.is_popup_visible);
            }
            _ => {}
        };
    }

    /// Toggle menu popup status
    pub fn chill_attempt(&mut self) {
        self.is_popup_visible = !self.is_popup_visible;
        // info!("__chill_attempt");
        match self.section {
            Section::Validators => {
                info!("__chill_attempt");
                self.validators.chill_attempt();
            }
            _ => {}
        };
    }
}
