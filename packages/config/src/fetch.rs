use crate::config::{NodeConfig, Source};
use crate::error::Error;
use reqwest::Client;
use tracing::info;

const BIN_NAME: &str = "suno";

/// Fetches validator stashes from a remote URL,
/// optionally using a GitHub PAT for private repositories.
pub async fn fetch_validators_from_source(source: &Source) -> Result<Vec<NodeConfig>, Error> {
    let client = Client::builder()
        .user_agent(format!("{}/{}", BIN_NAME, env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut request = client.get(source.url());

    if let Some(pat) = source.pat() {
        request = request.header("Authorization", format!("Bearer {}", pat))
    }

    let response = request.send().await?.error_for_status()?.text().await?;

    let v: Vec<NodeConfig> = response
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.parse::<NodeConfig>())
        .collect::<Result<Vec<_>, _>>()?;

    info!("{} stashes loaded from {}", v.len(), source.url());
    Ok(v)
}
