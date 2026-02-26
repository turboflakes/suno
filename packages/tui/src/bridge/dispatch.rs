use log::{error, info, warn};
use subxt::{blocks::ExtrinsicEvents, tx::TxProgress, tx::TxStatus, OnlineClient, SubstrateConfig};
use suno_actions::{Action, ChainAction, SystemAction, TxAction, ValidatorAction};
use suno_config::SupportedRuntime;
use suno_error::Error;
use suno_primitives::{AccountKey, Response};
use tokio::sync::mpsc::UnboundedSender;

pub fn dispatch_response_action(
    response: Response,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    match response {
        Response::Era(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEra(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::Epoch(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEpoch(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::TotalStaked(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalStaked(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::ActiveValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveValidators(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::ActiveNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveNominators(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::TotalValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalValidators(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::TotalNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalNominators(
                runtime.clone(),
                data.value,
            )))?;
        }
        Response::AuthorityStatus(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateStatus(
                account_key,
                data.value.status,
            )))?;
        }
        Response::AuthorityEraPoints(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateEraPoints(
                account_key,
                data.value.points,
            )))?;
        }
        Response::AuthorityPoints(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdatePoints(
                account_key,
                data.value.points,
            )))?;
        }
        Response::StakeOverview(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            if let Some(overview) = data.value.overview {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeOverview(
                    account_key,
                    overview,
                )))?;
            } else {
                warn!(
                    "No stake overview data found for {}",
                    account_key.to_string(),
                );
            }
        }
        Response::StakeLedger(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            if let Some(ledger) = data.value.ledger {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeLedger(
                    account_key,
                    ledger,
                )))?;
            } else {
                warn!("No stake ledger data found for {}", account_key.to_string(),);
            }
        }
        Response::Commission(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateCommission(
                account_key,
                data.value.commission.deconstruct(),
            )))?;
        }
        Response::Identity(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            if let Some(identity) = data.value.identity {
                tx.send(Action::Validator(ValidatorAction::UpdateIdentity(
                    account_key,
                    identity,
                )))?;
            } else {
                warn!("No identity data found for {}", account_key.to_string(),);
            }
        }
        Response::TxProgress(data) => {
            let response = data.value;
            spawn_process_transaction_progress(runtime, response, tx);
        }
        Response::TxSuccess => {
            let _ = tx.send(Action::Transaction(TxAction::Success));
        }
        Response::TxError(err) => {
            let _ = tx.send(Action::Transaction(TxAction::Error(err)));
        }
        Response::EventBonded(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::AddAmountToStakeLedger(
                account_key,
                data.value.amount,
            )))?;
        }
        Response::EventUnbonded(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime.clone(), data.value.account);
            tx.send(Action::Validator(ValidatorAction::SubChunkFromStakeLedger(
                account_key,
                data.value.chunk,
            )))?;
        }
        _ => {
            error!("Unhandled response type: {:?}", response);
        }
    }
    Ok(())
}

fn spawn_process_transaction_progress(
    runtime: &SupportedRuntime,
    progress: TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) {
    let runtime = runtime.clone();
    let mut progress = progress;
    let tx = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = process_transaction_progress(&runtime, &mut progress, &tx).await {
            let _ = tx.send(Action::System(SystemAction::Error(format!(
                "Dispatch error: {}",
                e
            ))));
        }
    });
}

async fn process_transaction_progress(
    runtime: &SupportedRuntime,
    progress: &mut TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    while let Some(status) = progress.next().await {
        match status? {
            TxStatus::Broadcasted => {
                let _ = tx.send(Action::Transaction(TxAction::Sent));
            }
            TxStatus::InBestBlock(block) => {
                let block_hash = block.block_hash();
                let _ = tx.send(Action::Transaction(TxAction::InBestBlock(block_hash)));
            }
            TxStatus::InFinalizedBlock(block) => {
                let block_hash = block.block_hash();
                let _ = tx.send(Action::Transaction(TxAction::InFinalizedBlock(block_hash)));
                info!(
                    "Transaction {:?} is finalized in block {:?}",
                    block.extrinsic_hash(),
                    block.block_hash()
                );

                match block.wait_for_success().await {
                    Ok(events) => {
                        let processed_events = process_extrinsic_events(events, &runtime);

                        for response in processed_events {
                            dispatch_response_action(response, runtime, tx)?;
                        }
                    }
                    Err(err) => {
                        error!("Transaction failed: {:?}", err);
                        let _ = tx.send(Action::Transaction(TxAction::Error(
                            "transaction failed".to_string(),
                        )));
                    }
                }
            }
            TxStatus::Error { message } => {
                let _ = tx.send(Action::Transaction(TxAction::Error(message)));
            }
            TxStatus::Invalid { message } => {
                let _ = tx.send(Action::Transaction(TxAction::Error(message)));
            }
            TxStatus::Dropped { message } => {
                let _ = tx.send(Action::Transaction(TxAction::Error(message)));
            }
            _ => {}
        }
    }
    Ok(())
}

fn process_extrinsic_events(
    events: ExtrinsicEvents<SubstrateConfig>,
    runtime: &SupportedRuntime,
) -> Vec<Response> {
    match runtime {
        SupportedRuntime::Paseo => {
            suno_paseo::handle_extrinsic_events(events).unwrap_or_else(|e| {
                error!("Error processing Paseo extrinsic events: {}", e);
                vec![]
            })
        }
        //  SupportedRuntime::AssetHubPolkadot => {
        //     suno_asset_hub_polkadot::handle_events(api, block_hash, events)
        //         .await
        //         .unwrap_or_else(|e| {
        //             error!("Error processing AssetHubPolkadot events: {}", e);
        //             vec![]
        //         })
        // }
        // SupportedRuntime::AssetHubKusama => {
        //     suno_asset_hub_kusama::handle_events(api, block_hash, events)
        //         .await
        //         .unwrap_or_else(|e| {
        //             error!("Error processing AssetHubKusama events: {}", e);
        //             vec![]
        //         })
        // }
        SupportedRuntime::AssetHubPaseo => suno_asset_hub_paseo::handle_extrinsic_events(events)
            .unwrap_or_else(|e| {
                error!("Error processing AssetHubPaseo extrinsic events: {}", e);
                vec![]
            }),
        // SupportedRuntime::AssetHubWestend => {
        //     suno_asset_hub_westend::handle_events(api, block_hash, events)
        //         .await
        //         .unwrap_or_else(|e| {
        //             error!("Error processing AssetHubWestend events: {}", e);
        //             vec![]
        //         })
        // }
        _ => {
            vec![]
        }
    }
}
