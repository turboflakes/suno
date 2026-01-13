use super::node_runtime;
use crate::{
    constants::{fetch_epoch_duration, fetch_expected_block_time},
    node_runtime::runtime_types::polkadot_primitives::v9::ValidatorIndex,
};
use std::collections::HashSet;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_error::Error;
use suno_events::Event;
use suno_primitives::{validator::ValidatorStatus, AccountKey, Epoch};

type Points = u32;
type Index = u64;
type BlockNumber = u32;

/// Fetch validator points at the specified block hash
pub async fn fetch_validator_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Points, Error> {
    let addr = node_runtime::storage()
        .staking_ah_client()
        .validator_points(stash.clone());

    Ok(api
        .storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .unwrap_or(0))
}

/// Fetch epoch data at the specified block hash
pub async fn fetch_epoch_data_event(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Event, Error> {
    let duration = fetch_epoch_duration(api)?;
    let block_time = fetch_expected_block_time(api)?;
    let (_, start) = fetch_epoch_start(api, block_hash).await?;
    let index = fetch_epoch_index(api, block_hash).await?;

    Ok(Event::NewEpoch(Epoch::new(
        index, start, duration, block_time,
    )))
}

/// Fetch validators authority status
pub async fn fetch_validators_authority_status_event(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    validator_keys: &Vec<AccountKey>,
) -> Result<Vec<Event>, Error> {
    let mut events: Vec<Event> = Vec::new();
    let validators = fetch_session_validators(api, block_hash).await?;
    let validator_indices = fetch_active_validator_indices(api, block_hash).await?;
    let validator_bytes: HashSet<[u8; 32]> = validator_keys.iter().map(|key| key.bytes()).collect();

    for (i, stash) in validators.iter().enumerate() {
        let bytes = *stash.as_ref();
        if validator_bytes.contains(&bytes) {
            if validator_indices.contains(&ValidatorIndex(i as u32)) {
                events.push(Event::AuthorityStatus(
                    bytes,
                    ValidatorStatus::ParaAuthority,
                ));
                continue;
            }
            events.push(Event::AuthorityStatus(bytes, ValidatorStatus::Authority));
        }
    }

    Ok(events)
}

/// Fetch babe epoch index at the specified block hash
async fn fetch_epoch_index(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Index, Error> {
    let addr = node_runtime::storage().babe().epoch_index();

    Ok(api
        .storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .unwrap_or(0))
}

/// Fetch babe epoch start at the specified block hash
async fn fetch_epoch_start(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<(BlockNumber, BlockNumber), Error> {
    let addr = node_runtime::storage().babe().epoch_start();

    Ok(api
        .storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .unwrap_or((0, 0)))
}

/// Fetch session validators at the specified block hash
async fn fetch_session_validators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Vec<AccountId32>, Error> {
    let addr = node_runtime::storage().session().validators();

    Ok(api
        .storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .unwrap_or_default())
}

/// Fetch active validator indices at the specified block hash
async fn fetch_active_validator_indices(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Vec<ValidatorIndex>, Error> {
    let addr = node_runtime::storage()
        .paras_shared()
        .active_validator_indices();

    Ok(api
        .storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .unwrap_or_default())
}
