use super::node_runtime;
use node_runtime::runtime_types::{
    bounded_collections::bounded_vec::BoundedVec, coretime_polkadot_runtime::ProxyType,
    coretime_polkadot_runtime::SessionKeys, frame_system::AccountInfo,
    pallet_balances::types::AccountData, pallet_proxy::ProxyDefinition,
};
use std::collections::HashMap;
use subxt::{utils::AccountId32, OnlineClientAtBlock};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::{balance::Balance, proxy::SupportedProxy, AccountKey, Response};

/// Fetch balance for a given stash at the specified block hash
pub async fn fetch_balance(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();

    let account_info = fetch_system_account(api, stash).await?;

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
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
    proxy: &AccountId32,
) -> Result<Vec<Response>, Error> {
    let mut responses: Vec<Response> = Vec::new();
    let account_bytes = *stash.as_ref();

    let (BoundedVec(proxies), _) = fetch_account_proxies(api, stash).await?;

    for def in proxies {
        if def.delegate == *proxy && def.proxy_type == ProxyType::NonTransfer {
            responses.push(Response::supported_proxy(
                account_bytes,
                SupportedProxy::NonTransfer,
            ));
        }
        if def.delegate == *proxy && def.proxy_type == ProxyType::Collator {
            responses.push(Response::supported_proxy(
                account_bytes,
                SupportedProxy::Collator,
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

/// Fetch proxies for a given account at the specified block hash
async fn fetch_account_proxies(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<
    (
        BoundedVec<ProxyDefinition<AccountId32, ProxyType, u32>>,
        u128,
    ),
    Error,
> {
    let addr = node_runtime::storage().proxy().proxies();

    let value = api
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
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<AccountInfo<u32, AccountData<u128>>, Error> {
    let addr = node_runtime::storage().system().account();

    let value = api
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

/// Fetch the current Aura authority set (session public keys) at the specified block hash
pub async fn fetch_aura_authorities(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().aura().authorities();

    let value = api
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    let authorities = value.0.iter().map(|public| public.0).collect();

    Ok(Response::aura_authorities(authorities))
}

/// Fetch the current session validators (collator stashes) at the specified block hash
pub async fn fetch_session_validators(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().session().validators();

    let value = api
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    let validators = value.iter().map(|stash| *stash.as_ref()).collect();

    Ok(Response::session_validators(validators))
}

/// Fetch the current session index at the specified block hash
pub async fn fetch_session_index(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().session().current_index();

    let value = api
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(Response::session_index(value))
}

/// Fetch the fixed invulnerable collator set at the specified block hash
pub async fn fetch_invulnerables(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Response, Error> {
    let addr = node_runtime::storage().collator_selection().invulnerables();

    let value = api
        .storage()
        .entry(addr)
        .boxed()?
        .fetch(())
        .await
        .boxed()?
        .decode()
        .boxed()?;

    let invulnerables = value.0.iter().map(|stash| *stash.as_ref()).collect();

    Ok(Response::invulnerables(invulnerables))
}

/// Fetch the last block authored by a given collator stash at the specified block hash
pub async fn fetch_collator_last_authored_block(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    let addr = node_runtime::storage()
        .collator_selection()
        .last_authored_block();

    let value = api
        .storage()
        .entry(addr)
        .boxed()?
        .fetch((*stash,))
        .await
        .boxed()?
        .decode()
        .boxed()?;

    Ok(Response::last_authored_block(account_bytes, value as u64))
}

/// Fetch collators queued keys
pub async fn fetch_collators_queued_keys(
    api: &OnlineClientAtBlock<CustomConfig>,
    collator_keys: &[AccountKey],
) -> Result<Vec<Response>, Error> {
    let mut responses: Vec<Response> = Vec::new();
    let queued_keys = fetch_session_queued_keys(api).await?;
    let mut collator_bytes: HashMap<[u8; 32], bool> = collator_keys
        .iter()
        .map(|key| (key.bytes(), false))
        .collect();

    for (stash, session_keys) in queued_keys.iter() {
        let bytes: [u8; 32] = *stash.as_ref();
        if let Some(found) = collator_bytes.get_mut(&bytes) {
            *found = true;
            responses.push(Response::collator_queued_keys(
                bytes,
                Some(session_keys.aura.0),
            ));
        }
    }

    // Emit None responses for collators not found in queued_keys
    for (bytes, found) in &collator_bytes {
        if !found {
            responses.push(Response::collator_queued_keys(*bytes, None));
        }
    }

    Ok(responses)
}

/// Fetch collator next session key
pub async fn fetch_collator_next_keys(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    if let Some(session_keys) = fetch_session_next_keys(api, stash).await? {
        return Ok(Response::collator_next_keys(
            account_bytes,
            Some(session_keys.aura.0),
        ));
    }

    Ok(Response::collator_next_keys(account_bytes, None))
}

/// Fetch queued keys for the next session at the specified block hash
async fn fetch_session_queued_keys(
    api: &OnlineClientAtBlock<CustomConfig>,
) -> Result<Vec<(AccountId32, SessionKeys)>, Error> {
    let addr = node_runtime::storage().session().queued_keys();

    let value = api
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

/// Fetch next session keys for a stash at the specified block hash
async fn fetch_session_next_keys(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<Option<SessionKeys>, Error> {
    let addr = node_runtime::storage().session().next_keys();

    let value = api
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
