mod error;
mod layer;
mod log;
mod visitor;

pub use crate::error::Error;
use crate::layer::TuiLayer;
pub use crate::log::{Log, LogEntry};
use std::result::Result;
use tokio::sync::mpsc;
use tracing_appender::non_blocking::WorkerGuard;
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

pub fn init_tui(tx: mpsc::UnboundedSender<LogEntry>) -> Result<WorkerGuard, Error> {
    let tui_layer = TuiLayer::new(tx);

    // TODO: get log filename path from CONFIG
    let file_appender = tracing_appender::rolling::never("logs", "suno.log");
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    let file_layer = fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true);

    Registry::default()
        .with(EnvFilter::from_default_env())
        .with(tui_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}
