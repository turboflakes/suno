use super::node_runtime;
use crate::error::Error;
use async_recursion::async_recursion;
use node_runtime::runtime_types::pallet_identity::types::Data;
use node_runtime::runtime_types::{
    pallet_identity::types::Registration, people_westend_runtime::people::IdentityInfo,
};
use std::result::Result;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_primitives::identity::Identity;

pub async fn fetch_display_name(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<String, Error> {
    if let Some(identity) = get_identity(api, block_hash, &stash, None).await? {
        return Ok(identity.to_string());
    } else {
        let s = &stash.to_string();
        Ok(format!("{}...{}", &s[..6], &s[s.len() - 6..]))
    }
}

#[async_recursion]
pub async fn get_identity(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
    sub_account_name: Option<String>,
) -> Result<Option<Identity>, Error> {
    // First, fetch the main identity data
    let identity_data = fetch_identity_of(api, block_hash, stash).await?;

    if let Some(registration) = identity_data {
        let parent = parse_identity_data(registration.info.display);
        let identity = match sub_account_name {
            Some(child) => Identity::with_name_and_sub(parent, child),
            None => Identity::with_name(parent),
        };
        return Ok(Some(identity));
    }

    // If no main identity, check if this is a sub-account
    let super_account = fetch_super_of(api, block_hash, stash).await?;

    if let Some((parent_account, sub_data)) = super_account {
        let sub_name = parse_identity_data(sub_data);
        return get_identity(api, block_hash, &parent_account, Some(sub_name.to_string())).await;
    }

    Ok(None)
}

async fn fetch_identity_of(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Option<Registration<u128, IdentityInfo>>, Error> {
    let addr = node_runtime::storage()
        .identity()
        .identity_of(stash.clone());

    let result = api.storage().at(block_hash).fetch(&addr).await?;
    Ok(result)
}

async fn fetch_super_of(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Option<(AccountId32, Data)>, Error> {
    let addr = node_runtime::storage().identity().super_of(stash.clone());

    let result = api.storage().at(block_hash).fetch(&addr).await?;
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
        _ => format!("???"),
    }
}

pub fn bytes_to_str(bytes: Vec<u8>) -> String {
    format!("{}", String::from_utf8_lossy(&bytes))
}
