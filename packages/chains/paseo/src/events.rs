use crate::node_runtime::session::events::NewSession;
use crate::storage::fetch_epoch_data;
use subxt::{events::Events, utils::H256, OnlineClient, SubstrateConfig};
use suno_error::Error;
use suno_primitives::event::Event;

pub async fn handle_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
) -> Result<Vec<Event>, Error> {
    let mut processed_events: Vec<Event> = Vec::new();
    for event in events.iter() {
        let event = event?;

        if let Some(_ev) = event.as_event::<NewSession>()? {
            let epoch = fetch_epoch_data(api, block_hash).await?;
            processed_events.push(Event::NewEpoch(epoch));
        }
    }
    Ok(processed_events)
}
