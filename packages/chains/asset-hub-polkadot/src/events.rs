use super::node_runtime;
use crate::storage::fetch_era_data;
use node_runtime::staking::events::{Chilled, EraPaid, ValidatorPrefsSet};
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

        if let Some(_ev) = event.as_event::<EraPaid>()? {
            let era = fetch_era_data(api, block_hash).await?;
            processed_events.push(Event::NewEra(era));
        } else if let Some(_ev) = event.as_event::<Chilled>()? {
        } else if let Some(_ev) = event.as_event::<ValidatorPrefsSet>()? {
        }
    }
    Ok(processed_events)
}
