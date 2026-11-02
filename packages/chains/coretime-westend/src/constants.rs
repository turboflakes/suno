use super::node_runtime;
use subxt::OnlineClientAtBlock;
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::Response;

/// Fetch Aura slot duration in milliseconds
pub async fn fetch_slot_duration(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Response, Error> {
    let addr = node_runtime::constants().aura().slot_duration();
    let value = api.constants().entry(&addr).boxed()?;

    Ok(Response::slot_duration(value))
}
