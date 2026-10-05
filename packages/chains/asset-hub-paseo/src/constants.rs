use super::node_runtime;
use subxt::OnlineClientAtBlock;
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::Response;

/// Fetch staking sessions per era
pub async fn fetch_sessions_per_era(api: &OnlineClientAtBlock<CustomConfig>) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().sessions_per_era();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(value)
}

/// Fetch Aura slot duration in milliseconds
pub async fn fetch_slot_duration(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Response, Error> {
    let addr = node_runtime::constants().aura().slot_duration();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(Response::slot_duration(value))
}

/// Fetch staking bonding duration
pub async fn fetch_bonding_duration(api: &OnlineClientAtBlock<CustomConfig>) -> Result<u32, Error> {
    let addr = node_runtime::constants().staking().bonding_duration();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(value)
}
