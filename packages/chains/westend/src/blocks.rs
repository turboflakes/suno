use crate::node_runtime::{
    proxy::{calls::Proxy, events::ProxyExecuted},
    runtime_types::{pallet_session::pallet::Call as SessionCall, westend_runtime::RuntimeCall},
    session::events::NewSession,
};
use crate::storage::{fetch_epoch_data, map_keys_from_session_keys};
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::{ExtrinsicEvents, Extrinsics},
    utils::{MultiAddress, H256},
    OnlineClient, SubstrateConfig,
};
use suno_error::{Error, ResultExt};
use suno_primitives::Response;

pub async fn process_runtime_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event.boxed()?;

        if event.is::<NewSession>() {
            let res = fetch_epoch_data(api, block_hash).await?;
            processed_events.push(res);
        }
        // else if let Some(ev) = event.decode_fields_as::<SessionKeysSet>().boxed()? {}
        // TODO: Event SessionKeysSet is currently not available in the runtime
        // so we process the extrinsic directly
    }
    Ok(processed_events)
}

pub async fn process_block_extrinsics(
    _api: &OnlineClient<SubstrateConfig>,
    _block_hash: H256,
    extrinsics: Extrinsics<'_, SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
) -> Result<Vec<Response>, Error> {
    let mut processed_extrinsics: Vec<Response> = Vec::new();
    for ext in extrinsics.find::<Proxy>() {
        let ext = ext.boxed()?;
        if let MultiAddress::Id(stash) = ext.real {
            let call = ext.call;
            if let RuntimeCall::Session(SessionCall::set_keys { keys, .. }) = call.as_ref() {
                let keys = map_keys_from_session_keys(keys);
                let account_bytes = *stash.as_ref();
                let res = Response::validator_next_keys(account_bytes, Some(keys));
                processed_extrinsics.push(res);
            }
        }
    }
    Ok(processed_extrinsics)
}

pub fn process_transaction_events(
    events: ExtrinsicEvents<SubstrateConfig>,
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
                } // TODO: handle DispatchError
                  // Err(err) => match err {
                  //     DispatchError::Module(data) => {
                  //         info!("ProxyExecuted event: {:?}", data.index);
                  //         let metadata = api.metadata();
                  //         if let Some(pallet) = metadata.pallet_by_index(data.index.into()) {
                  //             // Get the error variant based on the first byte of error_data
                  //             if let Some(error_index) = data.error.first() {
                  //                 if let Some(error) = pallet.error_variant_by_index(*error_index) {
                  //                     info!("Error: {:?}", error.docs.join("\n"));
                  //                 }
                  //             } else {
                  //                 return Err("Empty error data".into());
                  //             }
                  //         }
                  //     }
                  //     _ => {}
                  // },
            }
        }
    }
    Ok(processed_events)
}
