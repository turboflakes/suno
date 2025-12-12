use super::node_runtime;
use crate::error::Error;
use log::info;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_actions::{Action, ValidatorAction};
use suno_primitives::AccountKey;
use tokio::sync::mpsc::UnboundedSender;

type Points = u32;

/// Fetch and send validator points
pub async fn fas_validator_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    validator_key: AccountKey,
    tx: UnboundedSender<Action>,
) -> Result<(), Error> {
    let points = fetch_validator_points(api, block_hash, validator_key.stash()).await?;

    info!("__{}", points);
    tx.send(Action::Validator(ValidatorAction::UpdatePoints(
        validator_key,
        points,
    )))?;

    Ok(())
}

/// Fetch validator points at the specified block hash
async fn fetch_validator_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: AccountId32,
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
                "ValidatorPrefs not defined at block hash {block_hash:?} for stash {stash:?}"
            ))
        })
}
