use super::node_runtime;
use subxt::OnlineClientAtBlock;
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};

/// Fetch staking sessions per era
pub async fn fetch_sessions_per_era(api: &OnlineClientAtBlock<CustomConfig>) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().sessions_per_era();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(value)
}

/// Fetch staking bonding duration
pub async fn fetch_bonding_duration(api: &OnlineClientAtBlock<CustomConfig>) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().bonding_duration();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(value)
}
