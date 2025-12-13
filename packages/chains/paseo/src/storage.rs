use super::node_runtime;
use crate::error::Error;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};

type Points = u32;

/// Fetch validator points at the specified block hash
pub async fn fetch_validator_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: &AccountId32,
) -> Result<Points, Error> {
    let addr = node_runtime::storage()
        .staking_ah_client()
        .validator_points(stash.clone());

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "Validator points not defined at block hash {block_hash:?} for stash {stash:?}"
            ))
        })
}
