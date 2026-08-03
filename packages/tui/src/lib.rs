use crate::app::{App, AppResult};

pub mod app;
mod bridge;
mod event;
mod handler;
mod section;
mod tui;
mod ui;
mod utils;
mod widgets;

use suno_tracing::LogEntry;
use tokio::sync::mpsc;

pub async fn init(rx: mpsc::UnboundedReceiver<LogEntry>) -> AppResult<()> {
    // Create an application.
    let mut app = App::new(rx);

    // Run the application.
    app.run().await?;

    Ok(())
}
