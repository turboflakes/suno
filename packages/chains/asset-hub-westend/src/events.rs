use crate::storage::fetch_era_data;
use crate::{
    constants::fetch_bonding_duration,
    node_runtime::{
        proxy::events::ProxyExecuted,
        staking::events::{Bonded, Chilled, EraPaid, Unbonded, ValidatorPrefsSet},
    },
    storage::fetch_active_era_info,
};
use sp_arithmetic::Perbill;
use subxt::{blocks::ExtrinsicEvents, events::Events, utils::H256, OnlineClient, SubstrateConfig};
use suno_error::{Error, ResultExt};
use suno_primitives::{
    staking::{Chunk, ValidatorPrefs},
    Response,
};

pub async fn handle_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event.boxed()?;

        if let Some(_ev) = event.as_event::<EraPaid>().boxed()? {
            let response = fetch_era_data(api, block_hash).await?;
            processed_events.push(response);
        } else if let Some(ev) = event.as_event::<Bonded>().boxed()? {
            let account_bytes = *(ev.stash).as_ref();
            let response = Response::event_bonded(account_bytes, ev.amount);
            processed_events.push(response);
        } else if let Some(ev) = event.as_event::<Unbonded>().boxed()? {
            let era_info = fetch_active_era_info(api, block_hash).await?;
            let duration = fetch_bonding_duration(api)?;
            let chunk = Chunk::new(era_info.index + duration, ev.amount);
            let account_bytes = *(ev.stash).as_ref();
            let response = Response::event_unbonded(account_bytes, chunk);
            processed_events.push(response);
        } else if let Some(_ev) = event.as_event::<Chilled>().boxed()? {
        } else if let Some(ev) = event.as_event::<ValidatorPrefsSet>().boxed()? {
            let account_bytes = *(ev.stash).as_ref();
            let prefs =
                ValidatorPrefs::new(Perbill::from_parts(ev.prefs.commission.0), ev.prefs.blocked);
            let response = Response::validator_prefs_next(account_bytes, Some(prefs));
            processed_events.push(response);
        } // TODO: Event PayeeSet is currently not available
          // else if let Some(ev) = event.as_event::<PayeeSet>().boxed()? {}
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
