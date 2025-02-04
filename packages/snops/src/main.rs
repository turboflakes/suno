#[tokio::main]
async fn main() -> snops_tui::app::AppResult<()> {
    let _ = snops_tui::run().await?;

    Ok(())
}
