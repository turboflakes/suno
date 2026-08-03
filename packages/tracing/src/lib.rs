mod error;
mod layer;
mod log;
mod visitor;

pub use crate::error::Error;
use crate::layer::TuiLayer;
pub use crate::log::{Log, LogEntry};
use std::result::Result;
use tokio::sync::mpsc;
use tracing_subscriber::fmt;
use tracing_subscriber::{prelude::*, EnvFilter, Registry};

pub fn init_cli() -> Result<(), Error> {
    let cli_layer = fmt::layer().with_target(false).with_ansi(true);
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(cli_layer)
        .init();

    Ok(())
}

pub fn init_tui(tx: mpsc::UnboundedSender<LogEntry>) -> Result<(), Error> {
    let tui_layer = TuiLayer::new(tx);
    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(tui_layer)
        .init();

    Ok(())
}
