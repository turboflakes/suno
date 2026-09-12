use super::node_runtime;
use subxt::OnlineClientAtBlock;
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};

type Value = u64;

/// Fetch babe epoch duration in blocks
pub async fn fetch_epoch_duration(api: &OnlineClientAtBlock<CustomConfig>) -> Result<Value, Error> {
    let addr = node_runtime::constants().babe().epoch_duration();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(value)
}

/// Fetch babe expected block time in miliseconds
pub async fn fetch_expected_block_time(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Value, Error> {
    let addr = node_runtime::constants().babe().expected_block_time();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(value)
}
