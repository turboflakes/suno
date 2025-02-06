use snops_tui::app::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
    let _ = snops_tui::run().await?;

    Ok(())
}
