use crate::{
    constants::fetch_bonding_duration,
    node_runtime::{
        proxy::{calls::Proxy, events::ProxyExecuted},
        runtime_types::{
            asset_hub_polkadot_runtime::RuntimeCall,
            pallet_staking_async::pallet::pallet::Call as StakingCall,
            pallet_staking_async_rc_client::pallet::Call as StakingRcClientCall,
        },
        staking::events::{Bonded, Chilled, EraPaid, Unbonded, ValidatorPrefsSet, Withdrawn},
    },
    storage::{fetch_active_era_info, fetch_era_data, map_payee_from_reward_destination},
};
use sp_arithmetic::Perbill;
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::{ExtrinsicEvents, Extrinsics},
    utils::{MultiAddress, H256},
    OnlineClient, SubstrateConfig,
};
use suno_error::{Error, ResultExt};
use suno_primitives::{
    session::Keys,
    staking::{Chunk, ValidatorPrefs},
    Response,
};

pub async fn process_runtime_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
) -> Result<Vec<Response>, Error> {
    let mut processed_events: Vec<Response> = Vec::new();
    for event in events.iter() {
        let event = event.boxed()?;

        if event.is::<EraPaid>() {
            let response = fetch_era_data(api, block_hash).await?;
            processed_events.push(response);
        } else if let Some(ev) = event.decode_fields_as::<Bonded>() {
            let ev = ev.boxed()?;
            let account_bytes = *(ev.stash).as_ref();
            let response = Response::event_bonded(account_bytes, ev.amount);
            processed_events.push(response);
        } else if let Some(ev) = event.decode_fields_as::<Unbonded>() {
            let ev = ev.boxed()?;
            let era_info = fetch_active_era_info(api, block_hash).await?;
            let duration = fetch_bonding_duration(api, block_hash).await?;
            let chunk = Chunk::new(era_info.index + duration, ev.amount);
            let account_bytes = *(ev.stash).as_ref();
            let response = Response::event_unbonded(account_bytes, chunk);
            processed_events.push(response);
        } else if let Some(ev) = event.decode_fields_as::<Withdrawn>() {
            let ev = ev.boxed()?;
            let account_bytes = *(ev.stash).as_ref();
            let response = Response::event_withdrawn(account_bytes, ev.amount);
            processed_events.push(response);
        } else if let Some(ev) = event.decode_fields_as::<Chilled>() {
            let ev = ev.boxed()?;
            let account_bytes = *(ev.stash).as_ref();
            let prefs = ValidatorPrefs::new(Perbill::zero(), false);
            let response = Response::validator_prefs_next(account_bytes, Some(prefs));
            processed_events.push(response);
        } else if let Some(ev) = event.decode_fields_as::<ValidatorPrefsSet>() {
            let ev = ev.boxed()?;
            let account_bytes = *(ev.stash).as_ref();
            let prefs =
                ValidatorPrefs::new(Perbill::from_parts(ev.prefs.commission.0), ev.prefs.blocked);
            let response = Response::validator_prefs_next(account_bytes, Some(prefs));
            processed_events.push(response);
        } // else if let Some(ev) = event.as_event::<PayeeSet>().boxed()? {}
          // TODO: Event PayeeSet is currently not available in the runtime
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
            if let RuntimeCall::Staking(StakingCall::set_payee { payee }) = call.as_ref() {
                let payee = map_payee_from_reward_destination(payee.clone());
                let account_bytes = *stash.as_ref();
                let res = Response::validator_payee(account_bytes, payee);
                processed_extrinsics.push(res);
            } else if let RuntimeCall::StakingRcClient(StakingRcClientCall::set_keys {
                keys, ..
            }) = call.as_ref()
            {
                let keys = Keys::from_bytes(keys).boxed()?;
                let account_bytes = *stash.as_ref();
                let res = Response::validator_next_keys(account_bytes, Some(keys));
                processed_extrinsics.push(res);
            } else if let RuntimeCall::StakingRcClient(StakingRcClientCall::purge_keys { .. }) =
                call.as_ref()
            {
                let account_bytes = *stash.as_ref();
                let res = Response::validator_next_keys(account_bytes, None);
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
