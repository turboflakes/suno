use crate::node_runtime::session::events::NewSession;
use subxt::{events::Events, OnlineClientAtBlock};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::Response;

pub async fn process_runtime_events(
    _api: &OnlineClientAtBlock<CustomConfig>,
    events: Events<CustomConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event.boxed()?;

        if let Some(ev) = event.decode_fields_as::<NewSession>() {
            let ev = ev.boxed()?;
            let response = Response::session_index(ev.session_index);
            processed_events.push(response);
        }
    }
    Ok(processed_events)
}
