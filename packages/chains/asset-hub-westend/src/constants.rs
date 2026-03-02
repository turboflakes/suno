use super::node_runtime;
use subxt::{OnlineClient, SubstrateConfig};
use suno_error::{Error, ResultExt};

/// Fetch staking sessions per era
pub fn fetch_sessions_per_era(api: &OnlineClient<SubstrateConfig>) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().sessions_per_era();
    Ok(api.constants().at(&addr).boxed()?)
}

/// Fetch staking bonding duration
pub fn fetch_bonding_duration(api: &OnlineClient<SubstrateConfig>) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().bonding_duration();
    Ok(api.constants().at(&addr).boxed()?)
}
