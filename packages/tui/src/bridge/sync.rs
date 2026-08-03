use crate::bridge::{
    dispatch::dispatch_response_action, RuntimeCaller, RuntimeFetcher, RuntimeProcessor,
};
use futures::{stream, stream::StreamExt, Future};
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::Extrinsics,
    tx::{TransactionInBlock, TransactionProgress, TransactionStatus},
    utils::{AccountId32, H256},
    OnlineClient,
};
use subxt_signer::sr25519::Keypair;
use suno_actions::{Action, SystemAction, TxAction};
use suno_config::{CustomConfig, SupportedRuntime};
use suno_error::{Error, ResultExt};
use suno_primitives::{AccountKey, Response};
use tokio::sync::mpsc::UnboundedSender;
use tracing::error;

/// Default spawner for making asynchronous fetch requests.
struct DefaultSpawner {
    api: OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    tx: UnboundedSender<Action>,
}

impl DefaultSpawner {
    fn new(
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        runtime: SupportedRuntime,
        tx: &UnboundedSender<Action>,
    ) -> Self {
        Self {
            api: api.clone(),
            block_hash,
            runtime,
            tx: tx.clone(),
        }
    }

    fn spawn<F, Fut>(self, fetch_fn: F)
    where
        F: Fn(OnlineClient<CustomConfig>, H256) -> Fut + Send + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send,
    {
        let Self {
            api,
            block_hash,
            runtime,
            tx,
        } = self;

        tokio::spawn(async move {
            let result = fetch_fn(api, block_hash).await;
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
}

/// Validator spawner for making asynchronous fetch requests.
struct ValidatorSpawner {
    api: OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: Vec<AccountKey>,
    tx: UnboundedSender<Action>,
}

impl ValidatorSpawner {
    fn new(
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        runtime: SupportedRuntime,
        validator_keys: &[AccountKey],
        tx: &UnboundedSender<Action>,
    ) -> Self {
        Self {
            api: api.clone(),
            block_hash,
            runtime,
            validator_keys: validator_keys.to_vec(),
            tx: tx.clone(),
        }
    }

    fn spawn_unordered<F, Fut>(self, fetch_fn: F, n: usize)
    where
        F: Fn(OnlineClient<CustomConfig>, H256, AccountId32) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Response, Error>> + Send,
    {
        let Self {
            api,
            block_hash,
            runtime,
            validator_keys,
            tx,
        } = self;

        tokio::spawn(async move {
            let mut stream = stream::iter(validator_keys)
                .map(|key| {
                    let api = api.clone();
                    let stash = key.stash();
                    fetch_fn(api, block_hash, stash)
                })
                .buffer_unordered(n);

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

    fn spawn_unordered_multi<F, Fut>(self, fetch_fn: F, n: usize)
    where
        F: Fn(OnlineClient<CustomConfig>, H256, AccountId32) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Vec<Response>, Error>> + Send,
    {
        let Self {
            api,
            block_hash,
            runtime,
            validator_keys,
            tx,
        } = self;

        tokio::spawn(async move {
            let mut stream = stream::iter(validator_keys)
                .map(|key| {
                    let api = api.clone();
                    let stash = key.stash();
                    fetch_fn(api, block_hash, stash)
                })
                .buffer_unordered(n);

            while let Some(result) = stream.next().await {
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
            }
        });
    }

    fn spawn_batch<F, Fut>(self, fetch_fn: F)
    where
        F: Fn(OnlineClient<CustomConfig>, H256, Vec<AccountKey>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<Vec<Response>, Error>> + Send,
    {
        let Self {
            api,
            block_hash,
            runtime,
            validator_keys,
            tx,
        } = self;

        tokio::spawn(async move {
            let result = fetch_fn(api, block_hash, validator_keys).await;

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
}
// ----
// General chain fetcher tasks
// ----

pub fn spawn_fetch_era_data(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    DefaultSpawner::new(api, block_hash, runtime, tx)
        .spawn(move |api, bh| async move { runtime.fetch_era_data(&api, bh).await });
}

pub fn spawn_fetch_epoch_data(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    DefaultSpawner::new(api, block_hash, runtime, tx)
        .spawn(move |api, bh| async move { runtime.fetch_epoch_data(&api, bh).await });
}

pub fn spawn_fetch_total_staked(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    DefaultSpawner::new(api, block_hash, runtime, tx)
        .spawn(move |api, bh| async move { runtime.fetch_total_staked(&api, bh, era_index).await });
}

pub fn spawn_fetch_active_validators_count(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    DefaultSpawner::new(api, block_hash, runtime, tx).spawn(move |api, bh| async move {
        runtime
            .fetch_active_validators_count(&api, bh, era_index)
            .await
    });
}

pub fn spawn_fetch_active_nominators_count(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    era_index: u32,
    tx: &UnboundedSender<Action>,
) {
    DefaultSpawner::new(api, block_hash, runtime, tx).spawn(move |api, bh| async move {
        runtime
            .fetch_active_nominators_count(&api, bh, era_index)
            .await
    });
}

pub fn spawn_fetch_total_validators_count(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    DefaultSpawner::new(api, block_hash, runtime, tx)
        .spawn(move |api, bh| async move { runtime.fetch_total_validators_count(&api, bh).await });
}

pub fn spawn_fetch_total_nominators_count(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    DefaultSpawner::new(api, block_hash, runtime, tx)
        .spawn(move |api, bh| async move { runtime.fetch_total_nominators_count(&api, bh).await });
}

// ----
// Validator fetcher tasks
// ----
//
pub fn spawn_fetch_validators_era_points(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    era_index: u32,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_batch(
        move |api, bh, vk| async move {
            runtime
                .fetch_validators_era_points(&api, bh, era_index, &vk)
                .await
        },
    );
}

pub fn spawn_fetch_validators_authority_status(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_batch(
        move |api, bh, vk| async move {
            runtime
                .fetch_validators_authority_status(&api, bh, &vk)
                .await
        },
    );
}

pub fn spawn_fetch_validators_queued_keys(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_batch(
        move |api, bh, vk| async move { runtime.fetch_validators_queued_keys(&api, bh, &vk).await },
    );
}

pub fn spawn_fetch_validators_stake_overview(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    era_index: u32,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move {
            runtime
                .fetch_stake_overview(&api, bh, era_index, &stash)
                .await
        },
        3,
    );
}

pub fn spawn_fetch_validators_staking_ledger(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move { runtime.fetch_stake_ledger(&api, bh, &stash).await },
        3,
    );
}

pub fn spawn_fetch_validators_points(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move { runtime.fetch_validator_points(&api, bh, &stash).await },
        3,
    );
}

pub fn spawn_fetch_validators_prefs(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    era_index: u32,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move {
            runtime
                .fetch_validator_prefs(&api, bh, era_index, &stash)
                .await
        },
        3,
    );
}

pub fn spawn_fetch_validators_prefs_next(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move {
            runtime
                .fetch_validator_prefs_next(&api, bh, &stash)
                .await
        }, 3,
    );
}

pub fn spawn_fetch_validators_payee(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move { runtime.fetch_validator_payee(&api, bh, &stash).await },
        3,
    );
}

pub fn spawn_fetch_validators_next_keys(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move { runtime.fetch_validator_next_keys(&api, bh, &stash).await }, 3,
    );
}

pub fn spawn_fetch_validators_identity(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move { runtime.fetch_validator_identity(&api, bh, &stash).await }, 3,
    );
}

pub fn spawn_fetch_validators_proxy_status(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    proxy: &AccountId32,
    tx: &UnboundedSender<Action>,
) {
    let proxy = *proxy;
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered_multi(
        move |api, bh, stash| async move {
            runtime
                .fetch_and_validate_proxy_account(&api, bh, &stash, &proxy)
                .await
        },
        3,
    );
}

pub fn spawn_fetch_account_balance(
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    validator_keys: &[AccountKey],
    tx: &UnboundedSender<Action>,
) {
    ValidatorSpawner::new(api, block_hash, runtime, validator_keys, tx).spawn_unordered(
        move |api, bh, stash| async move { runtime.fetch_account_balance(&api, bh, &stash).await },
        3,
    );
}

// ----
// Caller tasks
// ----
pub fn spawn_sign_and_submit_call_data(
    api: &OnlineClient<CustomConfig>,
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
            .sign_and_submit_call_data(&api, &signer, &call_data)
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

pub fn spawn_submit_call_data_with_signature(
    api: &OnlineClient<CustomConfig>,
    runtime: SupportedRuntime,
    signer: &AccountId32,
    call_data: &[u8],
    signature: &[u8],
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let tx = tx.clone();
    let signer = *signer;
    let call_data = call_data.to_owned();
    let signature = signature.to_owned();

    let _ = tx.send(Action::Transaction(TxAction::Processing));

    tokio::spawn(async move {
        let result = runtime
            .submit_call_data_with_signature(&api, &signer, &call_data, &signature)
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

// pub fn spawn_submit_transaction(
//     api: &OnlineClient<CustomConfig>,
//     runtime: SupportedRuntime,
//     signed_tx: String,
//     tx: &UnboundedSender<Action>,
// ) {
//     let api = api.clone();
//     let tx = tx.clone();
//     let signed_tx = signed_tx.clone();

//     let _ = tx.send(Action::Transaction(TxAction::Processing));

//     tokio::spawn(async move {
//         let result = runtime.submit_transaction(&api, signed_tx).await;
//         match result {
//             Ok(response) => {
//                 if let Err(e) = dispatch_response_action(response, runtime, &tx) {
//                     let _ = tx.send(Action::System(SystemAction::Error(format!(
//                         "Dispatch error: {}",
//                         e
//                     ))));
//                 }
//             }
//             Err(e) => {
//                 let _ = tx.send(Action::System(SystemAction::Error(format!(
//                     "Signing error: {}",
//                     e
//                 ))));
//             }
//         }
//     });
// }

// ----
// Processor tasks
// ----
pub fn spawn_process_transaction_progress(
    runtime: SupportedRuntime,
    progress: TransactionProgress<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
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
    progress: &mut TransactionProgress<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    while let Some(status) = progress.next().await {
        let response = match status.boxed()? {
            TransactionStatus::Validated => Response::TxValidated,
            TransactionStatus::Broadcasted => Response::TxBroadcasted,
            TransactionStatus::NoLongerInBestBlock => Response::TxNoLongerInBestBlock,
            TransactionStatus::InBestBlock(in_block) => {
                let block_hash = in_block.block_hash();
                runtime.log_block_hash_explorer(block_hash);
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
    in_block: TransactionInBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
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
    in_block: &mut TransactionInBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
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
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    events: Events<CustomConfig>,
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
    api: &OnlineClient<CustomConfig>,
    block_hash: H256,
    runtime: SupportedRuntime,
    extrinsics: Extrinsics<'_, CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
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
