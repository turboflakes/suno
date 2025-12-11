use super::node_runtime;
use crate::error::Error;
pub use node_runtime::staking::storage::types::nominators::Nominators;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_actions::{Action, TxAction};
use tokio::sync::mpsc::UnboundedSender;

pub async fn fetch_initial_validator_data(
    api: OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: AccountId32,
    tx: UnboundedSender<Action>,
) -> Result<(), Error> {
    Ok(())
}

/// Fetch nominators at the specified block hash
pub async fn fetch_nominators(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: AccountId32,
) -> Result<Nominators, Error> {
    let addr = node_runtime::storage().staking().nominators(stash);

    api.storage()
        .at(block_hash)
        .fetch(&addr)
        .await?
        .ok_or_else(|| {
            Error::from(format!(
                "Nominators not defined at block hash {block_hash:?}"
            ))
        })
}
