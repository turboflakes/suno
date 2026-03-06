use crate::bridge::{
    dispatch::dispatch_response_action, RuntimeCaller, RuntimeFetcher, RuntimeProcessor,
};
use futures::{stream, stream::StreamExt};
use log::error;
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::Extrinsics,
    tx::{TransactionInBlock, TransactionProgress, TransactionStatus},
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use subxt_signer::sr25519::Keypair;
use suno_actions::{Action, SystemAction, TxAction};
use suno_config::SupportedRuntime;
use suno_error::{Error, ResultExt};
use suno_primitives::{AccountKey, Response};
use tokio::sync::mpsc::UnboundedSender;

const CONCURRENT_REQUESTS: usize = 3;

// ----
// Fetcher tasks
// ----

pub fn spawn_fetch_era_data(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_era_data(&api, block_hash).await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_epoch_data(&api, block_hash).await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_total_staked(&api, block_hash, era_index)
            .await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_active_validators_count(&api, block_hash, era_index)
            .await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_active_nominators_count(&api, block_hash, era_index)
            .await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_total_validators_count(&api, block_hash).await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime.fetch_total_nominators_count(&api, block_hash).await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    era_index: u32,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let validator_keys = validator_keys.to_vec();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_validators_era_points(&api, block_hash, era_index, &validator_keys)
            .await;
        match result {
            Ok(responses) => {
                for response in responses {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let validator_keys = validator_keys.to_vec();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_validators_authority_status(&api, block_hash, &validator_keys)
            .await;
        match result {
            Ok(responses) => {
                for response in responses {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

pub fn spawn_fetch_validators_queued_keys(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let validator_keys = validator_keys.to_vec();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .fetch_validators_queued_keys(&api, block_hash, &validator_keys)
            .await;
        match result {
            Ok(responses) => {
                for response in responses {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    era_index: u32,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
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
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let stash = key.stash();

                async move { runtime.fetch_stake_ledger(&api, block_hash, &stash).await }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
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
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

pub fn spawn_fetch_validators_prefs(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    era_index: u32,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_validator_prefs(&api, block_hash, era_index, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

pub fn spawn_fetch_validators_prefs_next(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_validator_prefs_next(&api, block_hash, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

pub fn spawn_fetch_validators_payee(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_validator_payee(&api, block_hash, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

pub fn spawn_fetch_validators_next_keys(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .fetch_validator_next_keys(&api, block_hash, &stash)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
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
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

pub fn spawn_fetch_validators_proxy_status(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    proxy: &AccountId32,
    tx: &UnboundedSender<Action>,
) {
    let validator_keys = validator_keys.to_vec();
    let proxy = proxy.clone();
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let mut stream = stream::iter(validator_keys)
            .map(|key| {
                let api = api.clone();
                let stash = key.stash();

                async move {
                    runtime
                        .validate_proxy_account(&api, block_hash, &stash, &proxy)
                        .await
                }
            })
            .buffer_unordered(CONCURRENT_REQUESTS);

        while let Some(result) = stream.next().await {
            match result {
                Ok(response) => {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
    runtime: SupportedRuntime,
    signer: &Keypair,
    call_data: &[u8],
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();
    let signer = signer.clone();
    let call_data = call_data.to_owned();

    let _ = tx.send(Action::Transaction(TxAction::Processing));

    tokio::spawn(async move {
        let result = runtime
            .sign_and_submit_call_data(&api, &signer, call_data)
            .await;
        match result {
            Ok(response) => {
                if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

// ----
// Processor tasks
// ----
pub fn spawn_process_transaction_progress(
    runtime: SupportedRuntime,
    progress: TransactionProgress<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) {
    let mut progress = progress;
    let tx = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = process_transaction_progress(runtime, &mut progress, &tx).await {
            let _ = tx.send(Action::System(SystemAction::Error(format!(
                "Transaction error: {}",
                e
            ))));
        }
    });
}

async fn process_transaction_progress(
    runtime: SupportedRuntime,
    progress: &mut TransactionProgress<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    while let Some(status) = progress.next().await {
        let response = match status.boxed()? {
            TransactionStatus::Validated => Response::TxValidated,
            TransactionStatus::Broadcasted => Response::TxBroadcasted,
            TransactionStatus::NoLongerInBestBlock => Response::TxNoLongerInBestBlock,
            TransactionStatus::InBestBlock(in_block) => {
                let block_hash = in_block.block_hash();
                Response::TxInBestBlock(block_hash)
            }
            TransactionStatus::InFinalizedBlock(in_block) => {
                let block_hash = in_block.block_hash();
                spawn_process_transaction_wait_for_success(runtime, in_block, tx);
                Response::TxInFinalizedBlock(block_hash)
            }
            TransactionStatus::Error { message } => Response::TxError(message),
            TransactionStatus::Invalid { message } => Response::TxError(message),
            TransactionStatus::Dropped { message } => Response::TxError(message),
        };
        if let Err(e) = dispatch_response_action(response, runtime, tx) {
            let _ = tx.send(Action::System(SystemAction::Error(format!(
                "Dispatch error: {}",
                e
            ))));
        }
    }
    Ok(())
}

fn spawn_process_transaction_wait_for_success(
    runtime: SupportedRuntime,
    in_block: TransactionInBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) {
    let mut in_block = in_block;
    let tx = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = process_transaction_wait_for_success(runtime, &mut in_block, &tx).await {
            let _ = tx.send(Action::System(SystemAction::Error(format!(
                "Transaction wait error: {}",
                e
            ))));
        }
    });
}

async fn process_transaction_wait_for_success(
    runtime: SupportedRuntime,
    in_block: &mut TransactionInBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    match in_block.wait_for_success().await {
        Ok(events) => {
            let result = runtime.process_transaction_events(events);

            match result {
                Ok(responses) => {
                    for response in responses {
                        if let Err(e) = dispatch_response_action(response, runtime, tx) {
                            let _ = tx.send(Action::System(SystemAction::Error(format!(
                                "Dispatch error: {}",
                                e
                            ))));
                        }
                    }
                }
                Err(e) => {
                    let _ = tx.send(Action::System(SystemAction::Error(format!(
                        "Process transaction error: {}",
                        e
                    ))));
                }
            }
        }
        Err(e) => {
            error!("Transaction failed: {:?}", e);
            let _ = tx.send(Action::Transaction(TxAction::Error(
                "transaction failed".to_string(),
            )));
        }
    }
    Ok(())
}

pub fn spawn_process_runtime_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .process_runtime_events(&api, block_hash, events)
            .await;
        match result {
            Ok(responses) => {
                for response in responses {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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

pub fn spawn_process_block_extrinsics(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    extrinsics: Extrinsics<'_, SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let extrinsics = extrinsics.into_owned();
    let tx = tx.clone();

    tokio::spawn(async move {
        let result = runtime
            .process_block_extrinsics(&api, block_hash, extrinsics)
            .await;
        match result {
            Ok(responses) => {
                for response in responses {
                    if let Err(e) = dispatch_response_action(response, runtime, &tx) {
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
