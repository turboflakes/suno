use crate::node_runtime::session::events::NewSession;
use crate::storage::fetch_epoch_data;
use subxt::{events::Events, utils::H256, OnlineClient, SubstrateConfig};
use suno_error::Error;
use suno_primitives::Response;

pub async fn handle_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event?;

        if let Some(_ev) = event.as_event::<NewSession>()? {
            let res = fetch_epoch_data(api, block_hash).await?;
            processed_events.push(res);
        }
    }
    Ok(processed_events)
}
