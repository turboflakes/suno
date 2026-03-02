use super::node_runtime;
use subxt::{OnlineClient, SubstrateConfig};
use suno_error::{Error, ResultExt};

type Value = u64;

/// Fetch babe epoch duration in blocks
pub fn fetch_epoch_duration(api: &OnlineClient<SubstrateConfig>) -> Result<Value, Error> {
    let addr = node_runtime::constants().babe().epoch_duration();
    Ok(api.constants().at(&addr).boxed()?)
}

/// Fetch babe expected block time in miliseconds
pub fn fetch_expected_block_time(api: &OnlineClient<SubstrateConfig>) -> Result<Value, Error> {
    let addr = node_runtime::constants().babe().expected_block_time();
    Ok(api.constants().at(&addr).boxed()?)
}
