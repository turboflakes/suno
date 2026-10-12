use crate::node_runtime::{
    proxy::{calls::Proxy, events::ProxyExecuted},
    runtime_types::{
        bridge_hub_polkadot_runtime::RuntimeCall,
        pallet_collator_selection::pallet::Call as CollatorSelectionCall,
        pallet_session::pallet::Call as SessionCall,
    },
    session::events::NewSession,
};
use crate::storage::{fetch_balance, fetch_candidacy_bond};
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
    api: &OnlineClientAtBlock<CustomConfig>,
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
                // Fetch candidacy bond and update collator deposit
                let deposit = fetch_candidacy_bond(api).await?;
                let res = Response::collator_deposit(account_bytes, deposit);
                processed_extrinsics.push(res);
                // Fetch account balance and update collator balance
                let res = fetch_balance(api, &stash).await?;
                processed_extrinsics.push(res);
            } else if let RuntimeCall::CollatorSelection(CollatorSelectionCall::leave_intent) =
                call.as_ref()
            {
                let account_bytes = *stash.as_ref();
                let res = Response::collator_status(account_bytes, CollatorStatus::Exiting);
                processed_extrinsics.push(res);
                // Set collator deposit to 0
                let res = Response::collator_deposit(account_bytes, 0);
                processed_extrinsics.push(res);
                // Fetch account balance and update collator balance
                let res = fetch_balance(api, &stash).await?;
                processed_extrinsics.push(res);
            } else if let RuntimeCall::CollatorSelection(CollatorSelectionCall::update_bond {
                new_deposit,
            }) = call.as_ref()
            {
                let account_bytes = *stash.as_ref();
                let res = Response::collator_deposit(account_bytes, *new_deposit);
                processed_extrinsics.push(res);
                // Fetch account balance and update collator balance
                let res = fetch_balance(api, &stash).await?;
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
