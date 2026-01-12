use crate::node_runtime::session::events::NewSession;
use crate::storage::fetch_epoch_data_event;
use subxt::{events::Events, utils::H256, OnlineClient, SubstrateConfig};
use suno_error::Error;
use suno_events::Event;

pub async fn handle_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
) -> Result<Vec<Event>, Error> {
    let mut processed_events: Vec<Event> = Vec::new();
    for event in events.iter() {
        let event = event?;

        if let Some(_ev) = event.as_event::<NewSession>()? {
            let ev = fetch_epoch_data_event(api, block_hash).await?;
            processed_events.push(ev);
        }
    }
    Ok(processed_events)
}
