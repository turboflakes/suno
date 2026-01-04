use super::node_runtime;
use subxt::{OnlineClient, SubstrateConfig};
use suno_error::Error;

/// Fetch staking sessions per era
pub fn fetch_sessions_per_era(api: &OnlineClient<SubstrateConfig>) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().sessions_per_era();
    Ok(api.constants().at(&addr)?)
}
