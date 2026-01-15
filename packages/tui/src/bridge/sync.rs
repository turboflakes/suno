use crate::bridge::{dispatch::dispatch_response_action, RuntimeFetcher};
use subxt::{utils::H256, OnlineClient, SubstrateConfig};
use suno_actions::{Action, SystemAction};
use suno_config::SupportedRuntime;
use tokio::sync::mpsc::UnboundedSender;

pub fn spawn_fetch_epoch_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_epoch_data(&api, block_hash).await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, &runtime, &tx) {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Dispatch error: {}",
                        e
                    ))));
                }
            }
            Err(e) => {
                let _ = tx.send(Action::System(SystemAction::Error(format!(
                    "Fetch error: {}",
                    e
                ))));
            }
        }
    });
}

pub fn spawn_fetch_total_staked(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_total_staked(&api, block_hash, era_index)
            .await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, &runtime, &tx) {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Dispatch error: {}",
                        e
                    ))));
                }
            }
            Err(e) => {
                let _ = tx.send(Action::System(SystemAction::Error(format!(
                    "Fetch error: {}",
                    e
                ))));
            }
        }
    });
}

pub fn spawn_fetch_active_validators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_active_validators_count(&api, block_hash, era_index)
            .await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, &runtime, &tx) {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Dispatch error: {}",
                        e
                    ))));
                }
            }
            Err(e) => {
                let _ = tx.send(Action::System(SystemAction::Error(format!(
                    "Fetch error: {}",
                    e
                ))));
            }
        }
    });
}

pub fn spawn_fetch_active_nominators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_active_nominators_count(&api, block_hash, era_index)
            .await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, &runtime, &tx) {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Dispatch error: {}",
                        e
                    ))));
                }
            }
            Err(e) => {
                let _ = tx.send(Action::System(SystemAction::Error(format!(
                    "Fetch error: {}",
                    e
                ))));
            }
        }
    });
}
