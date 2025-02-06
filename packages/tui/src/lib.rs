use log::LevelFilter;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use tui_logger::{init_logger, set_default_level};

use crate::{
    app::{App, AppResult},
    event::{Event, EventHandler},
    handler::handle_key_events,
    tui::Tui,
};

pub mod app;
mod config;
mod event;
mod handler;
mod tui;
mod ui;
mod utils;
mod widgets;

pub async fn run() -> AppResult<()> {
    // Initialize logs
    init_logger(LevelFilter::Info)?;
    set_default_level(LevelFilter::Info);

    // Create an application.
    let mut app = App::new();

    // Initialize the application.
    app.init().await;

    // Initialize the terminal user interface.
    let backend = CrosstermBackend::new(io::stdout());
    let terminal = Terminal::new(backend)?;
    let events = EventHandler::new(1000);
    let mut tui = Tui::new(terminal, events);
    tui.init()?;

    // Start the main loop.
    while app.running {
        // Render the user interface.
        tui.draw(&mut app)?;
        // Handle events.
        match tui.events.next().await? {
            Event::Tick => app.tick(),
            Event::Key(key_event) => handle_key_events(key_event, &mut app)?,
            Event::Mouse(_) => {}
            Event::Resize(_, _) => {}
        }
    }

    // Exit the user interface.
    tui.exit()?;
    Ok(())
}
