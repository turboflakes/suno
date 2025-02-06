use crate::config::SupportedRuntime;
use crate::widgets::chains::{ChainsListWidget, ConnectionState};
use crate::{
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};
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
    ScrollUp,
    ScrollDown,
    ChainConnection(SupportedRuntime, ConnectionState),
    Noop,
}

/// Application.
#[derive(Debug)]
pub struct App {
    /// Is the application running?
    pub running: bool,
    /// Holds the API clients for each supported runtime.
    pub chains: ChainsListWidget,
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
            chains: ChainsListWidget::default(),
            tx,
            rx,
        }
    }

    async fn init(&mut self) {
        let tx = self.tx.clone();
        self.chains.run(tx).await;
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
                Action::ScrollUp => self.chains.scroll_up(),
                Action::ScrollDown => self.chains.scroll_down(),
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
}
