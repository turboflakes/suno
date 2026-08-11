mod error;
mod layer;
mod log;
mod visitor;

pub use crate::error::Error;
use crate::layer::TuiLayer;
pub use crate::log::{Log, LogEntry};
use std::path::Path;
use std::result::Result;
use tokio::sync::mpsc;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt;
use tracing_subscriber::{prelude::*, Registry};

pub fn init_cli(file_path: Option<&str>) -> Result<Option<WorkerGuard>, Error> {
    let (file_layer, guard) = match file_path {
        Some(path) => {
            let (directory, filename) = parse_file_path(path)?;
            let (non_blocking, guard) = tracing_appender::non_blocking(
                tracing_appender::rolling::never(directory, filename),
            );
            let layer = fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .boxed();
            (Some(layer), Some(guard))
        }
        None => (None, None),
    };

    let cli_layer = fmt::layer().with_target(false).with_ansi(true);

    let default_level = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(default_level)
        .with(cli_layer)
        .with(file_layer)
        .init();

    Ok(guard)
}

pub fn init_tui(
    tx: mpsc::UnboundedSender<LogEntry>,
    file_path: Option<&str>,
) -> Result<Option<WorkerGuard>, Error> {
    let (file_layer, guard) = match file_path {
        Some(path) => {
            let (directory, filename) = parse_file_path(path)?;
            let (non_blocking, guard) = tracing_appender::non_blocking(
                tracing_appender::rolling::never(directory, filename),
            );
            let layer = fmt::layer()
                .with_writer(non_blocking)
                .with_ansi(false)
                .with_target(true)
                .boxed();
            (Some(layer), Some(guard))
        }
        None => (None, None),
    };

    let default_level = tracing_subscriber::EnvFilter::builder()
        .with_default_directive(tracing::Level::INFO.into())
        .from_env_lossy();

    Registry::default()
        .with(default_level)
        .with(TuiLayer::new(tx))
        .with(file_layer)
        .init();

    Ok(guard)
}

fn parse_file_path(path: &str) -> Result<(String, String), Error> {
    let path = Path::new(path);

    let directory = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| "logs".to_string());

    let filename = path
        .file_name()
        .ok_or(Error::InvalidFilename)?
        .to_string_lossy()
        .into_owned();

    Ok((directory, filename))
}
