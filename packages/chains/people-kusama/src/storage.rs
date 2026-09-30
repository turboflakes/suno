use super::node_runtime;
use async_recursion::async_recursion;
use node_runtime::runtime_types::pallet_identity::types::Data;
use node_runtime::runtime_types::{
    pallet_identity::types::Registration, people_kusama_runtime::people::IdentityInfo,
    people_kusama_runtime::SessionKeys,
};
use std::collections::HashMap;
use std::result::Result;
use subxt::{utils::AccountId32, OnlineClientAtBlock};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::{identity::Identity, AccountKey, Response};

pub async fn fetch_identity(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<Response, Error> {
    let account_bytes = *stash.as_ref();
    let identity = get_identity(api, stash, None).await?;
    Ok(Response::identity(account_bytes, identity))
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

#[async_recursion]
pub async fn get_identity(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
    sub_account_name: Option<String>,
) -> Result<Option<Identity>, Error> {
    // First, fetch the main identity data
    let identity_data = fetch_identity_of(api, stash).await?;

    if let Some(registration) = identity_data {
        let parent = parse_identity_data(registration.info.display);
        let identity = match sub_account_name {
            Some(child) => Identity::with_name_and_sub(parent, child),
            None => Identity::with_name(parent),
        };
        return Ok(Some(identity));
    }

    // If no main identity, check if this is a sub-account
    let super_account = fetch_super_of(api, stash).await?;

    if let Some((parent_account, sub_data)) = super_account {
        let sub_name = parse_identity_data(sub_data);
        return get_identity(api, &parent_account, Some(sub_name.to_string())).await;
    }

    Ok(None)
}

async fn fetch_identity_of(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<Option<Registration<u128, IdentityInfo>>, Error> {
    let addr = node_runtime::storage().identity().identity_of();

    let result = api
        .storage()
        .entry(addr)
        .boxed()?
        .try_fetch((*stash,))
        .await
        .boxed()?
        .map(|entry| entry.decode())
        .transpose()
        .boxed()?;

    Ok(result)
}

async fn fetch_super_of(
    api: &OnlineClientAtBlock<CustomConfig>,
    stash: &AccountId32,
) -> Result<Option<(AccountId32, Data)>, Error> {
    let addr = node_runtime::storage().identity().super_of();

    let result = api
        .storage()
        .entry(addr)
        .boxed()?
        .try_fetch((*stash,))
        .await
        .boxed()?
        .map(|entry| entry.decode())
        .transpose()
        .boxed()?;

    Ok(result)
}

fn parse_identity_data(data: Data) -> String {
    match data {
        Data::Raw0(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw1(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw2(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw3(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw4(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw5(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw6(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw7(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw8(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw9(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw10(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw11(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw12(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw13(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw14(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw15(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw16(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw17(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw18(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw19(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw20(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw21(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw22(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw23(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw24(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw25(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw26(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw27(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw28(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw29(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw30(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw31(bytes) => bytes_to_str(bytes.to_vec()),
        Data::Raw32(bytes) => bytes_to_str(bytes.to_vec()),
        _ => "???".to_string(),
    }
}

pub fn bytes_to_str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8_lossy(&bytes))
}
