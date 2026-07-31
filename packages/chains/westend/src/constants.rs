use super::node_runtime;
use subxt::{utils::H256, OnlineClient};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};

type Value = u64;

/// Fetch babe epoch duration in blocks
pub async fn fetch_epoch_duration(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
) -> Result<Value, Error> {
    let addr = node_runtime::constants().babe().epoch_duration();
    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at.constants().entry(&addr).boxed()?;

    Ok(value)
}

/// Fetch babe expected block time in miliseconds
pub async fn fetch_expected_block_time(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
) -> Result<Value, Error> {
    let addr = node_runtime::constants().babe().expected_block_time();
    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at.constants().entry(&addr).boxed()?;

    Ok(value)
}
