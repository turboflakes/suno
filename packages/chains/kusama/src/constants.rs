use super::node_runtime;
use subxt::{OnlineClient, SubstrateConfig};
use suno_error::Error;

type Value = u64;

/// Fetch babe epoch duration in blocks
pub fn fetch_epoch_duration(api: &OnlineClient<SubstrateConfig>) -> Result<Value, Error> {
    let addr = node_runtime::constants().babe().epoch_duration();
    Ok(api.constants().at(&addr)?)
}
