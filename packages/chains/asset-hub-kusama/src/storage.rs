use crate::constants::fetch_sessions_per_era;
use crate::node_runtime;
use crate::node_runtime::runtime_types::{
    asset_hub_kusama_runtime::ProxyType,
    bounded_collections::bounded_vec::BoundedVec,
    frame_system::AccountInfo,
    pallet_balances::types::AccountData,
    pallet_proxy::ProxyDefinition,
    pallet_staking_async::{
        ledger::StakingLedger, ActiveEraInfo, EraRewardPoints, Nominations, RewardDestination,
        ValidatorPrefs,
    },
    sp_staking::PagedExposureMetadata,
};
use crate::utils::map_reward_destination;
use sp_arithmetic::{Perbill, Permill};
use std::collections::HashSet;
use subxt::{
    ext::futures::StreamExt,
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_error::{Error, ResultExt};
use suno_primitives::{
    balance::Balance,
    node_account::get_account_bytes_from_storage_key,
    proxy::SupportedProxy,
    staking,
    staking::{Chunk, Payee},
    AccountKey, Response,
};

/// Fetch balance for a given stash at the specified block hash
pub async fn fetch_balance(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();

    let account_info = fetch_system_account(api, block_hash, stash).await?;

    Ok(Response::balance(
        account_bytes,
        Balance::new(
            account_info.data.free,
            account_info.data.frozen,
            account_info.data.reserved,
        ),
    ))
}

/// Fetch and validate a proxy account for a given stash at the specified block hash
pub async fn fetch_and_validate_proxy_account(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
    proxy: &AccountId32,
) -> Result<Vec<Response>, Error> {
    let mut responses: Vec<Response> = Vec::new();
    let account_bytes = *stash.as_ref();

    let (BoundedVec(proxies), _) = fetch_account_proxies(api, block_hash, stash).await?;

    for def in proxies {
        if def.delegate == *proxy && def.proxy_type == ProxyType::Staking {
            responses.push(Response::supported_proxy(
                account_bytes,
                SupportedProxy::Staking,
            ));
        }
        if def.delegate == *proxy && def.proxy_type == ProxyType::StakingOperator {
            responses.push(Response::supported_proxy(
                account_bytes,
                SupportedProxy::StakingOperator,
            ));
        }
    }

    if responses.is_empty() {
        responses.push(Response::supported_proxy(
            account_bytes,
            SupportedProxy::None,
        ));
    }

    Ok(responses)
}

/// Fetch validator prefs
pub async fn fetch_validator_prefs(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    if let Some(data) = fetch_eras_validator_prefs(api, block_hash, era, stash).await? {
        let prefs =
            staking::ValidatorPrefs::new(Perbill::from_parts(data.commission.0), data.blocked);
        return Ok(Response::validator_prefs(account_bytes, Some(prefs)));
    }

    Ok(Response::validator_prefs(account_bytes, None))
}

/// Fetch validator next prefs
pub async fn fetch_validator_prefs_next(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    let data = fetch_validators(api, block_hash, stash).await?;
    let prefs = staking::ValidatorPrefs::new(Perbill::from_parts(data.commission.0), data.blocked);
    Ok(Response::validator_prefs_next(account_bytes, Some(prefs)))
}

/// Fetch validator stake overview
pub async fn fetch_validator_stake_overview(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    if let Some(data) = fetch_eras_stakers_overview(api, block_hash, era, stash).await? {
        let stake_overview =
            staking::StakeOverview::new(data.own, data.total, data.nominator_count);
        return Ok(Response::stake_overview(
            account_bytes,
            Some(stake_overview),
        ));
    }
    Ok(Response::stake_overview(account_bytes, None))
}

pub async fn fetch_validator_staking_ledger(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();

    if let Some(data) = fetch_staking_ledger(api, block_hash, stash).await? {
        let mut unbounding: Vec<Chunk> = Vec::new();
        let BoundedVec(unlocking) = data.unlocking;
        for chunk in unlocking {
            unbounding.push(Chunk::new(chunk.era, chunk.value));
        }
        let stake_ledger = staking::StakeLedger::new(data.active, data.total, unbounding);
        return Ok(Response::stake_ledger(account_bytes, Some(stake_ledger)));
    }
    Ok(Response::stake_ledger(account_bytes, None))
}

/// Fetch validators era points
pub async fn fetch_validators_era_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
    validator_keys: &[AccountKey],
) -> Result<Vec<Response>, Error> {
    let mut responses: Vec<Response> = Vec::new();
    if let Some(reward_points) = fetch_era_reward_points(api, block_hash, era).await? {
        let validator_bytes: HashSet<[u8; 32]> =
            validator_keys.iter().map(|key| key.bytes()).collect();

        for (stash, points) in reward_points.individual.0.iter() {
            let bytes = *stash.as_ref();
            if validator_bytes.contains(&bytes) {
                responses.push(Response::authority_era_points(bytes, *points));
            }
        }

        return Ok(responses);
    }

    Ok(responses)
}

/// Fetch era data at the specified block hash
pub async fn fetch_era_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Response, Error> {
    let sessions_per_era = fetch_sessions_per_era(api, block_hash).await?;
    let era_info = fetch_active_era_info(api, block_hash).await?;
    let BoundedVec(bonded_eras) = fetch_bonded_eras(api, block_hash).await?;
    let start_session: u64 = bonded_eras
        .iter()
        .find(|b| b.0 == era_info.index)
        .map(|c| c.1 as u64)
        .unwrap_or(0);
    let era = staking::Era::new(
        era_info.index,
        era_info.start.unwrap_or(0),
        start_session,
        sessions_per_era,
    );
    Ok(Response::era(era))
}

/// Fetch active validators and nominators at the specified block hash
pub async fn fetch_active_nominators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().staking().eras_stakers_overview();

    let mut validators_set = HashSet::<[u8; 32]>::new();
    let api_at = api.at_block(block_hash).await.boxed()?;
    let mut iter = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .iter((era,))
        .await
        .boxed()?;
    while let Some(Ok(storage_kv)) = iter.next().await {
        let account_id = get_account_bytes_from_storage_key(storage_kv.key_bytes());
        validators_set.insert(account_id);
    }

    let mut nominators_count = 0u32;
    let addr = node_runtime::storage().staking().nominators();
    let api_at = api.at_block(block_hash).await.boxed()?;
    let mut iter = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .iter(())
        .await
        .boxed()?;
    while let Some(Ok(storage_kv)) = iter.next().await {
        // Check if any of the nominator's targets is in the validators_set
        let nominations = storage_kv.value().decode().boxed()?;
        if nominations
            .targets
            .0
            .iter()
            .any(|acc| validators_set.contains(&acc.0))
        {
            nominators_count += 1;
        }
    }

    Ok(Response::active_nominators(nominators_count))
}

/// Fetch active validators at the specified block hash
pub async fn fetch_active_validators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().staking().eras_stakers_overview();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let iter = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .iter((era,))
        .await
        .boxed()?;
    let count = iter.count().await;

    Ok(Response::active_validators(count as u32))
}

/// Fetch total validators at the specified block hash
pub async fn fetch_total_validators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().staking().counter_for_validators();

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

    Ok(Response::total_validators(value))
}

/// Fetch total nominators at the specified block hash
pub async fn fetch_total_nominators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().staking().counter_for_nominators();

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

    Ok(Response::total_nominators(value))
}

/// Fetch total total staked for a specific era at the specified block hash
pub async fn fetch_total_staked(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
) -> Result<Response, Error> {
    let total_issuance = fetch_total_issuance(api, block_hash).await?;
    let inactive_issuance = fetch_inactive_issuance(api, block_hash).await?;
    let total_staked = fetch_eras_total_stake(api, block_hash, era).await?;

    let active_issuance = total_issuance.saturating_sub(inactive_issuance);

    if active_issuance == 0 {
        return Ok(Response::total_staked(Permill::zero()));
    }

    Ok(Response::total_staked(Permill::from_rational(
        total_staked,
        active_issuance,
    )))
}

/// Fetch validator payee for a specific stash at the specified block hash
pub async fn fetch_validator_payee(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    let destination = fetch_payee(api, block_hash, stash).await?;
    let payee = map_reward_destination(destination);

    Ok(Response::validator_payee(account_bytes, payee))
}

//
// -----------------------------------------
//

/// Fetch bonded eras at the specified block hash
async fn fetch_bonded_eras(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<BoundedVec<(u32, u32)>, Error> {
    let addr = node_runtime::storage().staking().bonded_eras();

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

/// Fetch eras total stake for a specific era at the specified block hash
async fn fetch_eras_total_stake(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
) -> Result<u128, Error> {
    let addr = node_runtime::storage().staking().eras_total_stake();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((era,))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch total issuance for at the specified block hash
async fn fetch_total_issuance(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<u128, Error> {
    let addr = node_runtime::storage().balances().total_issuance();

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

/// Fetch inactive issuance for at the specified block hash
async fn fetch_inactive_issuance(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<u128, Error> {
    let addr = node_runtime::storage().balances().inactive_issuance();

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

/// Fetch validator prefs at the specified block hash
async fn fetch_validators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<ValidatorPrefs, Error> {
    let addr = node_runtime::storage().staking().validators();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((*stash,))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch validator prefs at the specified block hash and era
async fn fetch_eras_validator_prefs(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
    stash: &AccountId32,
) -> Result<Option<ValidatorPrefs>, Error> {
    let addr = node_runtime::storage().staking().eras_validator_prefs();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .try_fetch((era, *stash))
        .await
        .boxed()?
        .map(|entry| entry.decode())
        .transpose()
        .boxed()?;

    Ok(value)
}

/// Fetch staking ledger at the specified block hash
async fn fetch_staking_ledger(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Option<StakingLedger>, Error> {
    let addr = node_runtime::storage().staking().ledger();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .try_fetch((*stash,))
        .await
        .boxed()?
        .map(|entry| entry.decode())
        .transpose()
        .boxed()?;

    Ok(value)
}

/// Fetch active era at the specified block hash
pub async fn fetch_active_era_info(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
) -> Result<ActiveEraInfo, Error> {
    let addr = node_runtime::storage().staking().active_era();

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

/// Fetch era reward points at the specified block hash
async fn fetch_era_reward_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
) -> Result<Option<EraRewardPoints>, Error> {
    let addr = node_runtime::storage().staking().eras_reward_points();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .try_fetch((era,))
        .await
        .boxed()?
        .map(|entry| entry.decode())
        .transpose()
        .boxed()?;

    Ok(value)
}

/// Fetch eras_stakers_overview at the specified block hash for the given era and stash
async fn fetch_eras_stakers_overview(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    era: u32,
    stash: &AccountId32,
) -> Result<Option<PagedExposureMetadata<u128>>, Error> {
    let addr = node_runtime::storage().staking().eras_stakers_overview();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .try_fetch((era, *stash))
        .await
        .boxed()?
        .map(|entry| entry.decode())
        .transpose()
        .boxed()?;

    Ok(value)
}

/// Fetch nominators at the specified block hash
async fn _fetch_nominators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Nominations, Error> {
    let addr = node_runtime::storage().staking().nominators();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((*stash,))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch payee at the specified block hash
pub async fn fetch_payee(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<RewardDestination<AccountId32>, Error> {
    let addr = node_runtime::storage().staking().payee();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((*stash,))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch proxies for a given account at the specified block hash
async fn fetch_account_proxies(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<
    (
        BoundedVec<ProxyDefinition<AccountId32, ProxyType, u32>>,
        u128,
    ),
    Error,
> {
    let addr = node_runtime::storage().proxy().proxies();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((*stash,))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

/// Fetch balance for a given account at the specified block hash
async fn fetch_system_account(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<AccountInfo<u32, AccountData<u128>>, Error> {
    let addr = node_runtime::storage().system().account();

    let api_at = api.at_block(block_hash).await.boxed()?;
    let value = api_at
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((*stash,))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(value)
}

// Helper function to map RewardDestination to Payee
pub fn map_payee_from_reward_destination(dest: RewardDestination<AccountId32>) -> Payee {
    match dest {
        RewardDestination::None | RewardDestination::Controller => Payee::None,
        RewardDestination::Account(account) => Payee::Account(account),
        RewardDestination::Stash => Payee::Stash,
        RewardDestination::Staked => Payee::Staked,
    }
}
