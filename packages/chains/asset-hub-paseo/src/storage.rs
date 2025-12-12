use super::node_runtime;
use crate::error::Error;
use node_runtime::{
    runtime_types::{pallet_staking_async::ValidatorPrefs, sp_arithmetic::per_things::Perbill},
    staking::storage::types::nominators::Nominators,
};
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_actions::{Action, ValidatorAction};
use suno_primitives::AccountKey;
use tokio::sync::mpsc::UnboundedSender;

/// Fetch and send initial validator data
pub async fn fas_validator_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    validator_key: AccountKey,
    tx: UnboundedSender<Action>,
) -> Result<(), Error> {
    let validator_prefs = fetch_validator_prefs(api, block_hash, validator_key.stash()).await?;
    let Perbill(commission) = validator_prefs.commission;

    tx.send(Action::Validator(ValidatorAction::UpdateChangeCommission(
        validator_key,
        commission,
    )))?;

    Ok(())
}

/// Fetch validator prefs at the specified block hash
async fn fetch_validator_prefs(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    stash: AccountId32,
) -> Result<ValidatorPrefs, Error> {
    let addr = node_runtime::storage().staking().validators(stash.clone());

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

/// Fetch nominators at the specified block hash
async fn fetch_nominators(
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
                "Nominators not defined at block hash {block_hash:?} for stash {stash:?}"
            ))
        })
}
