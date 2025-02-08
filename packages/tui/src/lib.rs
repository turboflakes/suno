use log::LevelFilter;
use tui_logger::{init_logger, set_default_level};

use crate::app::{App, AppResult};

pub mod app;
mod config;
mod event;
mod handler;
mod node_account;
mod tui;
mod ui;
mod utils;
mod widgets;

pub async fn start() -> AppResult<()> {
    // Initialize logs
    init_logger(LevelFilter::Debug)?;
    set_default_level(LevelFilter::Debug);

    // Create an application.
    let mut app = App::new();

    // Run the application.
    app.run().await?;

    Ok(())
}
