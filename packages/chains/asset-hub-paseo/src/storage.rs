use super::node_runtime;
use crate::error::Error;
use node_runtime::{
    runtime_types::pallet_staking_async::{ActiveEraInfo, EraRewardPoints, ValidatorPrefs},
    staking::storage::types::nominators::Nominators,
};
use std::collections::{HashMap, HashSet};
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_primitives::AccountKey;

/// Fetch validator commission
pub async fn fetch_validator_commission(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<u32, Error> {
    let prefs = fetch_validator_prefs(api, block_hash, stash).await?;
    Ok(prefs.commission.0)
}

/// Fetch validators era points
pub async fn fetch_validators_era_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    validator_keys: &Vec<AccountKey>,
) -> Result<HashMap<[u8; 32], u32>, Error> {
    let active_era_info = fetch_active_era_info(api, block_hash).await?;

    if let Some(reward_points) =
        fetch_era_reward_points(api, block_hash, active_era_info.index).await?
    {
        let validator_bytes: HashSet<[u8; 32]> =
            validator_keys.iter().map(|key| key.bytes()).collect();

        let points_map: HashMap<[u8; 32], u32> = reward_points
            .individual
            .0
            .iter()
            .filter_map(|(stash, points)| {
                let bytes = *stash.as_ref();
                if validator_bytes.contains(&bytes) {
                    Some((bytes, *points))
                } else {
                    None
                }
            })
            .collect();

        return Ok(points_map);
    }

    Ok(HashMap::new())
}

/// Fetch validator prefs at the specified block hash
async fn fetch_validator_prefs(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<ValidatorPrefs, Error> {
    let addr = node_runtime::storage().staking().validators(stash.clone());

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "ValidatorPrefs not defined at block hash {block_hash:?} for stash {stash}"
            ))
        })
}

/// Fetch active era at the specified block hash
async fn fetch_active_era_info(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<ActiveEraInfo, Error> {
    let addr = node_runtime::storage().staking().active_era();

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "Active era not defined at block hash {block_hash:?}"
            ))
        })
}

/// Fetch era reward points at the specified block hash
async fn fetch_era_reward_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
) -> Result<Option<EraRewardPoints>, Error> {
    let addr = node_runtime::storage().staking().eras_reward_points(era);

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await
        .map_err(|e| e.into())
}

/// Fetch nominators at the specified block hash
async fn _fetch_nominators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: AccountId32,
) -> Result<Nominators, Error> {
    let addr = node_runtime::storage().staking().nominators(stash.clone());

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "Nominators not defined at block hash {block_hash:?} for stash {stash}"
            ))
        })
}
