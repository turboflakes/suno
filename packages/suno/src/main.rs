use std::result::Result;
use suno_config::{Subcommand, CONFIG};
use suno_error::Error;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load config
    let config = CONFIG.clone();

    // Run subcommands or start TUI
    match config.subcommand() {
        Some(Subcommand::Update { version }) => {
            suno_tracing::init_cli()?;
            suno_update::run(version.as_deref()).await?;
        }
        _ => {
            let (tx, rx) = mpsc::unbounded_channel();
            suno_tracing::init_tui(tx)?;
            suno_tui::init(rx).await?;
        }
    }

    Ok(())
}
