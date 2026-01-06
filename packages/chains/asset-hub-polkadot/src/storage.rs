use super::node_runtime;
use crate::constants::fetch_sessions_per_era;
use node_runtime::{
    runtime_types::{
        bounded_collections::bounded_vec::BoundedVec,
        pallet_staking_async::{
            ledger::StakingLedger, ActiveEraInfo, EraRewardPoints, ValidatorPrefs,
        },
    },
    staking::storage::types::{eras_stakers_overview::ErasStakersOverview, nominators::Nominators},
};
use std::collections::{HashMap, HashSet};
use subxt::{
    ext::futures::StreamExt,
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_error::Error;
use suno_primitives::{
    staking::{Era, StakeLedger, StakeOverview},
    AccountKey,
};

/// Fetch validator commission
pub async fn fetch_validator_commission(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<u32, Error> {
    let prefs = fetch_validator_prefs(api, block_hash, stash).await?;
    Ok(prefs.commission.0)
}

/// Fetch validators stake information
pub async fn fetch_validators_stake_overview(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    validator_keys: &Vec<AccountKey>,
) -> Result<HashMap<[u8; 32], StakeOverview>, Error> {
    let active_era_info = fetch_active_era_info(api, block_hash).await?;

    let mut stakes_map = HashMap::new();

    for key in validator_keys {
        if let Some(data) =
            fetch_validator_stake_overview(api, block_hash, active_era_info.index, &key.stash())
                .await?
        {
            stakes_map.insert(key.bytes(), data);
        }
    }

    Ok(stakes_map)
}

/// Fetch validator stake overview
pub async fn fetch_validator_stake_overview(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
    stash: &AccountId32,
) -> Result<Option<StakeOverview>, Error> {
    if let Some(data) = fetch_eras_stakers_overview(api, block_hash, era, stash).await? {
        let stake_overview = StakeOverview::new(data.own, data.total, data.nominator_count);
        return Ok(Some(stake_overview));
    }
    Ok(None)
}

pub async fn fetch_validator_staking_ledger(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Option<StakeLedger>, Error> {
    if let Some(data) = fetch_staking_ledger(api, block_hash, stash).await? {
        let stake_ledger = StakeLedger::new(data.active, data.total);
        return Ok(Some(stake_ledger));
    }
    Ok(None)
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

/// Fetch era data at the specified block hash
pub async fn fetch_era_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Era, Error> {
    let sessions_per_era = fetch_sessions_per_era(api)?;
    let era_info = fetch_active_era_info(api, block_hash).await?;
    let BoundedVec(bonded_eras) = fetch_bonded_eras(api, block_hash).await?;
    let start_session: u64 = bonded_eras
        .iter()
        .find(|b| b.0 == era_info.index)
        .map(|c| c.1 as u64)
        .unwrap_or(0);

    Ok(Era::new(
        era_info.index,
        era_info.start.unwrap_or(0),
        start_session,
        sessions_per_era,
    ))
}

/// Fetch active validators at the specified block hash
pub async fn fetch_active_validators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<u32, Error> {
    let active_era_info = fetch_active_era_info(api, block_hash).await?;

    let addr = node_runtime::storage()
        .staking()
        .eras_stakers_overview_iter1(active_era_info.index);

    let iter = api.storage().at(block_hash).iter(addr).await?;
    let count = iter.count().await;

    Ok(count as u32)
}

/// Fetch total validators at the specified block hash
pub async fn fetch_total_validators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<u32, Error> {
    let addr = node_runtime::storage().staking().counter_for_validators();

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "Total validators not defined at block hash {block_hash:?}"
            ))
        })
}

/// Fetch total nominators at the specified block hash
pub async fn fetch_total_nominators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<u32, Error> {
    let addr = node_runtime::storage().staking().counter_for_nominators();

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "Total nominators not defined at block hash {block_hash:?}"
            ))
        })
}

/// Fetch bonded eras at the specified block hash
async fn fetch_bonded_eras(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<BoundedVec<(u32, u32)>, Error> {
    let addr = node_runtime::storage().staking().bonded_eras();

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "BondedEras not defined at block hash {block_hash:?}"
            ))
        })
}

/// Fetch eras total stake for a specific era at the specified block hash
async fn fetch_eras_total_stake(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
) -> Result<u128, Error> {
    let addr = node_runtime::storage().staking().eras_total_stake(era);

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "Counter for nominators not defined at block hash {block_hash:?}"
            ))
        })
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

/// Fetch staking ledger at the specified block hash
async fn fetch_staking_ledger(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Option<StakingLedger>, Error> {
    let addr = node_runtime::storage().staking().ledger(stash.clone());

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await
        .map_err(|e| e.into())
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

/// Fetch eras_stakers_overview at the specified block hash for the given era and stash
async fn fetch_eras_stakers_overview(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
    stash: &AccountId32,
) -> Result<Option<ErasStakersOverview>, Error> {
    let addr = node_runtime::storage()
        .staking()
        .eras_stakers_overview(era, stash.clone());

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
