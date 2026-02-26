use crate::bridge::{dispatch::dispatch_response_action, RuntimeFetcher};
use futures::{stream, stream::StreamExt};
use subxt::{utils::H256, OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::Keypair;
use suno_actions::{Action, SystemAction, TxAction};
use suno_config::SupportedRuntime;
use suno_error::Error;
use suno_primitives::{tx::payload_from_bytes, AccountKey, Response};
use tokio::sync::mpsc::UnboundedSender;

const CONCURRENT_REQUESTS: usize = 3;

// ----
// Fetcher tasks
// ----

pub fn spawn_fetch_era_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_era_data(&api, block_hash).await;
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

pub fn spawn_fetch_total_validators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_total_validators_count(&api, block_hash).await;
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

pub fn spawn_fetch_total_nominators_count(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_total_nominators_count(&api, block_hash).await;
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

pub fn spawn_fetch_validators_era_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    era_index: u32,
    validator_keys: &Vec<AccountKey>,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let validator_keys = validator_keys.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_validators_era_points(&api, block_hash, era_index, &validator_keys)
            .await;
        match result {
            Ok(responses) => {
                for response in responses {
                    if let Err(e) = dispatch_response_action(response, &runtime, &tx) {
                        let _ = tx.send(Action::System(SystemAction::Error(format!(
                            "Dispatch error: {}",
                            e
                        ))));
                    }
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

pub fn spawn_fetch_validators_authority_status(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: &Vec<AccountKey>,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let validator_keys = validator_keys.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_validators_authority_status(&api, block_hash, &validator_keys)
            .await;
        match result {
            Ok(responses) => {
                for response in responses {
                    if let Err(e) = dispatch_response_action(response, &runtime, &tx) {
                        let _ = tx.send(Action::System(SystemAction::Error(format!(
                            "Dispatch error: {}",
                            e
                        ))));
                    }
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

pub fn spawn_fetch_validators_stake_overview(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    era_index: u32,
    validator_keys: &Vec<AccountKey>,
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.clone();
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let runtime = runtime.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_stake_overview(&api, block_hash, era_index, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
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
        }
    });
}

pub fn spawn_fetch_validators_staking_ledger(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: &Vec<AccountKey>,
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.clone();
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let runtime = runtime.clone();
                let stash = key.stash();

                async move { runtime.fetch_stake_ledger(&api, block_hash, &stash).await }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
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
        }
    });
}

pub fn spawn_fetch_validators_points(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: &Vec<AccountKey>,
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.clone();
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let runtime = runtime.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_validator_points(&api, block_hash, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
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
        }
    });
}

pub fn spawn_fetch_validators_commission(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: &Vec<AccountKey>,
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.clone();
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let runtime = runtime.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_validator_commission(&api, block_hash, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
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
        }
    });
}

pub fn spawn_fetch_validators_identity(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: &Vec<AccountKey>,
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.clone();
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let runtime = runtime.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_validator_identity(&api, block_hash, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
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
        }
    });
}

// ----
// Caller tasks
// ----
pub fn spawn_sign_and_submit(
    api: &OnlineClient<SubstrateConfig>,
    runtime: &SupportedRuntime,
    signer: &Keypair,
    call_data: &Vec<u8>,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();
    let signer = signer.clone();
    let call_data = call_data.clone();

    let _ = tx.send(Action::Transaction(TxAction::Processing));

    tokio::spawn(async move {
        let result = sign_and_submit_call_data(&api, &signer, call_data).await;
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
                    "Signing error: {}",
                    e
                ))));
            }
        }
    });
}

async fn sign_and_submit_call_data(
    api: &OnlineClient<SubstrateConfig>,
    proxy_signer: &Keypair,
    call_data: Vec<u8>,
) -> Result<Response, Error> {
    let payload = payload_from_bytes(call_data);

    let response = api
        .tx()
        .sign_and_submit_then_watch_default(&payload, proxy_signer)
        .await?;

    Ok(Response::transaction_progress(response))
}
