use suno_tui::app::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    suno_tui::start().await?;

    Ok(())
}
