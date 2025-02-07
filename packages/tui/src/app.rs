use crate::config::SupportedRuntime;
use crate::widgets::{
    chains::{ChainsListWidget, ConnectionState},
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
enum Window {
    #[default]
    Chains,
    Validators,
    Collators,
    Rpcs,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// The current window.
    pub window: Window,
    /// Holds the API clients for each supported runtime.
    pub chains: ChainsListWidget,
    /// Holds the validators list for the selected chain.
    pub validators: ValidatorsListWidget,
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
            window: Window::Chains,
            chains: ChainsListWidget::default(),
            validators: ValidatorsListWidget::default(),
            tx,
            rx,
        }
    }

    async fn init(&mut self) {
        let tx = self.tx.clone();
        self.chains.run(tx).await;
        if let Some(chain) = self.chains.get_selected() {
            let tx = self.tx.clone();
            self.validators.on_chain_selected(chain, tx);
        }
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
            info!("__{:?}", self.window);
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
        match self.window {
            Window::Chains => {
                self.chains.scroll_up();
                if let Some(chain) = self.chains.get_selected() {
                    let tx = self.tx.clone();
                    self.validators.on_chain_selected(chain, tx);
                }
            }
            _ => {}
        };
    }

    /// Moves row selection down.
    pub fn scroll_down(&mut self) {
        match self.window {
            Window::Chains => {
                self.chains.scroll_down();
                if let Some(chain) = self.chains.get_selected() {
                    let tx = self.tx.clone();
                    self.validators.on_chain_selected(chain, tx);
                }
            }
            _ => {}
        };
    }

    /// Moves the active window up.
    pub fn window_up(&mut self) {
        self.window = match self.window {
            Window::Chains => Window::Rpcs,
            Window::Validators => Window::Chains,
            Window::Collators => Window::Validators,
            Window::Rpcs => Window::Collators,
        };
        self.chains.set_active(self.window == Window::Chains);
        self.validators
            .set_active(self.window == Window::Validators);
    }

    /// Moves the active window down.
    pub fn window_down(&mut self) {
        self.window = match self.window {
            Window::Chains => Window::Validators,
            Window::Validators => Window::Collators,
            Window::Collators => Window::Rpcs,
            Window::Rpcs => Window::Chains,
        };
        self.chains.set_active(self.window == Window::Chains);
        self.validators
            .set_active(self.window == Window::Validators);
    }
}
