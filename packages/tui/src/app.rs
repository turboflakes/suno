use crate::config::{Features, SupportedRuntime, CONFIG};
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
    WindowUp,
    WindowDown,
    ScrollUp,
    ScrollDown,
    ChainConnection(SupportedRuntime, ConnectionState),
    Noop,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Section {
    #[default]
    Chains,
    Validators,
    Collators,
    Rpcs,
}

impl Section {
    fn up(&self, features: &Features) -> Self {
        use Section::*;

        match self {
            Chains => Self::up_from_chains(features),
            Validators => Chains,
            Collators => Self::up_from_collators(features),
            Rpcs => Self::up_from_rpcs(features),
        }
    }

    fn up_from_chains(features: &Features) -> Self {
        if features.enable_rpcs {
            Self::Rpcs
        } else if features.enable_collators {
            Self::Collators
        } else {
            Self::Validators
        }
    }

    fn up_from_collators(features: &Features) -> Self {
        if features.enable_validators {
            Self::Validators
        } else {
            Self::Chains
        }
    }

    fn up_from_rpcs(features: &Features) -> Self {
        if features.enable_collators {
            Self::Collators
        } else if features.enable_validators {
            Self::Validators
        } else {
            Self::Chains
        }
    }

    fn down(&self, features: &Features) -> Self {
        use Section::*;

        match self {
            Chains => Self::down_from_chains(features),
            Validators => Self::down_from_validators(features),
            Collators => Self::down_from_collators(features),
            Rpcs => Chains,
        }
    }

    fn down_from_chains(features: &Features) -> Self {
        if features.enable_validators {
            Self::Validators
        } else if features.enable_collators {
            Self::Collators
        } else {
            Self::Rpcs
        }
    }

    fn down_from_validators(features: &Features) -> Self {
        if features.enable_collators {
            Self::Collators
        } else if features.enable_rpcs {
            Self::Rpcs
        } else {
            Self::Chains
        }
    }

    fn down_from_collators(features: &Features) -> Self {
        if features.enable_rpcs {
            Self::Rpcs
        } else {
            Self::Chains
        }
    }
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
                Action::WindowUp => self.window_up(),
                Action::WindowDown => self.window_down(),
                Action::ScrollUp => self.scroll_up(),
                Action::ScrollDown => self.scroll_down(),
                Action::ChainConnection(runtime, connection) => {
                    self.chains.set_connection_state(runtime, connection)
                }
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
    pub fn scroll_up(&mut self) {
        match self.section {
            Section::Chains => {
                self.chains.scroll_up();
            }
            Section::Validators => {
                self.validators.scroll_up();
            }
            Section::Collators => {
                self.collators.scroll_up();
            }
            _ => {}
        };
    }

    /// Moves row selection down.
    pub fn scroll_down(&mut self) {
        match self.section {
            Section::Chains => {
                self.chains.scroll_down();
            }
            Section::Validators => {
                self.validators.scroll_down();
            }
            Section::Collators => {
                self.collators.scroll_down();
            }
            _ => {}
        };
    }

    /// Moves the active window up.
    pub fn window_up(&mut self) {
        let config = CONFIG.clone();
        self.section = self.section.up(&config.features);
        self.chains.set_active(self.section == Section::Chains);
        self.validators
            .set_active(self.section == Section::Validators);
        self.collators
            .set_active(self.section == Section::Collators);
    }

    /// Moves the active window down.
    pub fn window_down(&mut self) {
        let config = CONFIG.clone();
        self.section = self.section.down(&config.features);
        self.chains.set_active(self.section == Section::Chains);
        self.validators
            .set_active(self.section == Section::Validators);
        self.collators
            .set_active(self.section == Section::Collators);
    }
}
