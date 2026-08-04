use crate::config::NodeConfig;
use crate::error::Error;
use reqwest::Client;
use tracing::info;

/// Fetches validator stashes from a remote URL,
/// optionally using a GitHub PAT for private repositories.
pub async fn fetch_validators_from_url(
    url: &str,
    github_pat: Option<&str>,
) -> Result<Vec<NodeConfig>, Error> {
    let client = Client::builder()
        .user_agent(format!("suno/{}", env!("CARGO_PKG_VERSION")))
        .build()?;

    let mut request = client.get(url);

    if let Some(pat) = github_pat {
        request = request
            .header("Authorization", format!("Bearer {}", pat))
            .header("Accept", "application/vnd.github.raw");
    }

    let response = request.send().await?.error_for_status()?.text().await?;

    let v: Vec<NodeConfig> = response
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| s.parse::<NodeConfig>())
        .collect::<Result<Vec<_>, _>>()?;

    info!("{} stashes loaded from {}", v.len(), url);
    Ok(v)
}
