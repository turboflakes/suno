use std::result::Result;
use suno_config::{Subcommand, CONFIG};
use suno_error::Error;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load config
    let config = CONFIG.clone();
    let file_path = config.logs_file_path();

    // Run subcommands or start TUI
    match config.subcommand() {
        Some(Subcommand::Update { version }) => {
            let _guard = suno_tracing::init_cli(file_path)?;
            suno_update::run_update(version.as_deref()).await?;
        }
        _ => {
            let (tx, rx) = mpsc::unbounded_channel();
            let _guard = suno_tracing::init_tui(tx, file_path)?;
            suno_tui::init(rx).await?;
        }
    }

    Ok(())
}
