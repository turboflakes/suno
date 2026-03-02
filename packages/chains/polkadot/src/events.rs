use crate::node_runtime::{proxy::events::ProxyExecuted, session::events::NewSession};
use crate::storage::fetch_epoch_data;
use subxt::{blocks::ExtrinsicEvents, events::Events, utils::H256, OnlineClient, SubstrateConfig};
use suno_error::{Error, ResultExt};
use suno_primitives::Response;

pub async fn handle_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event.boxed()?;

        if let Some(_ev) = event.as_event::<NewSession>().boxed()? {
            let res = fetch_epoch_data(api, block_hash).await?;
            processed_events.push(res);
        }
    }
    Ok(processed_events)
}

pub fn handle_extrinsic_events(
    events: ExtrinsicEvents<SubstrateConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event.boxed()?;

        if let Some(ev) = event.as_event::<ProxyExecuted>().boxed()? {
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
