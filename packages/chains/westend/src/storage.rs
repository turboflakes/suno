use super::node_runtime;
use crate::{
    constants::{fetch_epoch_duration, fetch_expected_block_time},
    node_runtime::runtime_types::{
        polkadot_primitives::v9::ValidatorIndex,
        polkadot_primitives::v9::{
            assignment_app::Public as AssignmentPublic, validator_app::Public as ValidatorPublic,
        },
        sp_authority_discovery::app::Public as AuthorityDiscoveryPublic,
        sp_consensus_babe::app::Public as BabePublic,
        sp_consensus_beefy::ecdsa_crypto::Public as BeefyPublic,
        sp_consensus_grandpa::app::Public as GrandpaPublic,
        westend_runtime::SessionKeys,
    },
};
use std::collections::{HashMap, HashSet};
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_error::{Error, ResultExt};
use suno_primitives::{session::Keys, validator::ValidatorStatus, AccountKey, Epoch, Response};

type Index = u64;
type BlockNumber = u32;

/// Fetch validator points at the specified block hash
pub async fn fetch_validator_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();

    let addr = node_runtime::storage()
        .staking_ah_client()
        .validator_points();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((stash.clone(),))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(Response::authority_points(account_bytes, value))
}

/// Fetch epoch data at the specified block hash
pub async fn fetch_epoch_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Response, Error> {
    let duration = fetch_epoch_duration(api, block_hash).await?;
    let block_time = fetch_expected_block_time(api, block_hash).await?;
    let (_, start) = fetch_epoch_start(api, block_hash).await?;
    let index = fetch_epoch_index(api, block_hash).await?;

    Ok(Response::epoch(Epoch::new(
        index, start, duration, block_time,
    )))
}

/// Fetch validators authority status
pub async fn fetch_validators_authority_status(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    validator_keys: &[AccountKey],
) -> Result<Vec<Response>, Error> {
    let mut responses: Vec<Response> = Vec::new();
    let validators = fetch_session_validators(api, block_hash).await?;
    let validator_indices = fetch_active_validator_indices(api, block_hash).await?;
    let validator_bytes: HashSet<[u8; 32]> = validator_keys.iter().map(|key| key.bytes()).collect();

    for (i, stash) in validators.iter().enumerate() {
        let bytes = *stash.as_ref();
        if validator_bytes.contains(&bytes) {
            if validator_indices.contains(&ValidatorIndex(i as u32)) {
                responses.push(Response::authority_status(
                    bytes,
                    ValidatorStatus::ParaAuthority,
                ));
                continue;
            }
            responses.push(Response::authority_status(
                bytes,
                ValidatorStatus::Authority,
            ));
        }
    }

    Ok(responses)
}

/// Fetch validators queued keys
pub async fn fetch_validators_queued_keys(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    validator_keys: &[AccountKey],
) -> Result<Vec<Response>, Error> {
    let mut responses: Vec<Response> = Vec::new();
    let queued_keys = fetch_session_queued_keys(api, block_hash).await?;
    let mut validator_bytes: HashMap<[u8; 32], bool> = validator_keys
        .iter()
        .map(|key| (key.bytes(), false))
        .collect();

    for (stash, session_keys) in queued_keys.iter() {
        let bytes: [u8; 32] = *stash.as_ref();
        if let Some(found) = validator_bytes.get_mut(&bytes) {
            *found = true;
            let keys = map_keys_from_session_keys(session_keys);
            responses.push(Response::validator_queued_keys(bytes, Some(keys)));
        }
    }

    // Emit None responses for validators not found in queued_keys
    for (bytes, found) in &validator_bytes {
        if !found {
            responses.push(Response::validator_queued_keys(*bytes, None));
        }
    }

    Ok(responses)
}

/// Fetch validator next session key
pub async fn fetch_validator_next_keys(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    if let Some(session_keys) = fetch_session_next_keys(api, block_hash, stash).await? {
        let keys = map_keys_from_session_keys(&session_keys);
        return Ok(Response::validator_next_keys(account_bytes, Some(keys)));
    };

    Ok(Response::validator_next_keys(account_bytes, None))
}

/// Fetch babe epoch index at the specified block hash
async fn fetch_epoch_index(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Index, Error> {
    let addr = node_runtime::storage().babe().epoch_index();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch babe epoch start at the specified block hash
async fn fetch_epoch_start(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<(BlockNumber, BlockNumber), Error> {
    let addr = node_runtime::storage().babe().epoch_start();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch session validators at the specified block hash
async fn fetch_session_validators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Vec<AccountId32>, Error> {
    let addr = node_runtime::storage().session().validators();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch active validator indices at the specified block hash
async fn fetch_active_validator_indices(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Vec<ValidatorIndex>, Error> {
    let addr = node_runtime::storage()
        .paras_shared()
        .active_validator_indices();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch queued keys for the next session for a stash at the specified block hash
async fn fetch_session_next_keys(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Option<SessionKeys>, Error> {
    let addr = node_runtime::storage().session().next_keys();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .try_fetch((stash.clone(),))
        .await
        .boxed()?
        .map(|entry| entry.decode())
        .transpose()
        .boxed()?;

    Ok(value)
}

/// Fetch queued keys for the next session at the specified block hash
async fn fetch_session_queued_keys(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Vec<(AccountId32, SessionKeys)>, Error> {
    let addr = node_runtime::storage().session().queued_keys();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

// Helper function to map SessionKeys to Keys
pub fn map_keys_from_session_keys(session_keys: &SessionKeys) -> Keys {
    let GrandpaPublic(grandpa) = session_keys.grandpa;
    let BabePublic(babe) = session_keys.babe;
    let ValidatorPublic(para) = session_keys.para_validator;
    let AssignmentPublic(assi) = session_keys.para_assignment;
    let AuthorityDiscoveryPublic(auth) = session_keys.authority_discovery;
    let BeefyPublic(beef) = session_keys.beefy;
    Keys::new(grandpa, babe, para, assi, auth, beef)
}
