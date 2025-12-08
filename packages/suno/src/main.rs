use suno_tui::app::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    let _ = suno_tui::start().await?;

    Ok(())
}
