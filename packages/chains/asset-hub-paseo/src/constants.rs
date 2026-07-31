use super::node_runtime;
use subxt::{utils::H256, OnlineClient};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};

/// Fetch staking sessions per era
pub async fn fetch_sessions_per_era(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().sessions_per_era();
    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at.constants().entry(&addr).boxed()?;

    Ok(value)
}

/// Fetch staking bonding duration
pub async fn fetch_bonding_duration(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().bonding_duration();
    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at.constants().entry(&addr).boxed()?;

    Ok(value)
}
