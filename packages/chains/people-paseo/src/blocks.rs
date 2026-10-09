use crate::node_runtime::{
    proxy::{calls::Proxy, events::ProxyExecuted},
    runtime_types::{
        pallet_collator_selection::pallet::Call as CollatorSelectionCall,
        pallet_session::pallet::Call as SessionCall, people_paseo_runtime::RuntimeCall,
    },
    session::events::NewSession,
};
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::{ExtrinsicEvents, Extrinsics},
    utils::MultiAddress,
    OnlineClientAtBlock,
};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::{collator::CollatorStatus, Response};

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

pub async fn process_block_extrinsics(
    _api: &OnlineClientAtBlock<CustomConfig>,
    extrinsics: Extrinsics<'_, CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
) -> Result<Vec<Response>, Error> {
    let mut processed_extrinsics: Vec<Response> = Vec::new();
    for ext in extrinsics.find::<Proxy>() {
        let ext = ext.boxed()?;
        if let MultiAddress::Id(stash) = ext.real {
            let call = ext.call;
            if let RuntimeCall::Session(SessionCall::set_keys { keys, .. }) = call.as_ref() {
                let account_bytes = *stash.as_ref();
                let res = Response::collator_next_keys(account_bytes, Some(keys.aura.0));
                processed_extrinsics.push(res);
            } else if let RuntimeCall::Session(SessionCall::purge_keys) = call.as_ref() {
                let account_bytes = *stash.as_ref();
                let res = Response::collator_next_keys(account_bytes, None);
                processed_extrinsics.push(res);
            } else if let RuntimeCall::CollatorSelection(
                CollatorSelectionCall::register_as_candidate,
            ) = call.as_ref()
            {
                let account_bytes = *stash.as_ref();
                let res = Response::collator_status(account_bytes, CollatorStatus::Candidate);
                processed_extrinsics.push(res);
            }
        }
    }
    Ok(processed_extrinsics)
}

pub fn process_transaction_events(
    events: ExtrinsicEvents<CustomConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event.boxed()?;

        if let Some(ev) = event.decode_fields_as::<ProxyExecuted>() {
            let ev = ev.boxed()?;
            match ev.result {
                Ok(_) => {
                    processed_events.push(Response::TxSuccess);
                }
                Err(err) => {
                    processed_events.push(Response::TxError(format!(
                        "ProxyExecuted with error {:?}",
                        err
                    )));
                }
            }
        }
    }
    Ok(processed_events)
}
