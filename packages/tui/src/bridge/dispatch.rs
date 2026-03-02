use log::{error, info, warn};
use subxt::{blocks::ExtrinsicEvents, tx::TxProgress, tx::TxStatus, OnlineClient, SubstrateConfig};
use suno_actions::{Action, ChainAction, SystemAction, TxAction, ValidatorAction};
use suno_config::SupportedRuntime;
use suno_error::{Error, ResultExt};
use suno_primitives::{AccountKey, Response};
use tokio::sync::mpsc::UnboundedSender;

pub fn dispatch_response_action(
    response: Response,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    match response {
        Response::Era(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEra(runtime, data.value)))
                .boxed()?;
        }
        Response::Epoch(data) => {
            tx.send(Action::Chain(ChainAction::UpdateEpoch(runtime, data.value)))
                .boxed()?;
        }
        Response::TotalStaked(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalStaked(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::ActiveValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveValidators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::ActiveNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateActiveNominators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::TotalValidators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalValidators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::TotalNominators(data) => {
            tx.send(Action::Chain(ChainAction::UpdateTotalNominators(
                runtime, data.value,
            )))
            .boxed()?;
        }
        Response::AuthorityStatus(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateStatus(
                account_key,
                data.value.status,
            )))
            .boxed()?;
        }
        Response::AuthorityEraPoints(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdateEraPoints(
                account_key,
                data.value.points,
            )))
            .boxed()?;
        }
        Response::AuthorityPoints(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdatePoints(
                account_key,
                data.value.points,
            )))
            .boxed()?;
        }
        Response::StakeOverview(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(overview) = data.value.overview {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeOverview(
                    account_key,
                    overview,
                )))
                .boxed()?;
            } else {
                warn!("No stake overview data found for {}", account_key,);
            }
        }
        Response::StakeLedger(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(ledger) = data.value.ledger {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeLedger(
                    account_key,
                    ledger,
                )))
                .boxed()?;
            } else {
                warn!("No stake ledger data found for {}", account_key,);
            }
        }
        Response::ValidatorPrefs(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(prefs) = data.value.prefs {
                tx.send(Action::Validator(ValidatorAction::UpdateValidatorPrefs(
                    account_key,
                    prefs,
                )))
                .boxed()?;
            } else {
                warn!("No validator prefs data found for {}", account_key,);
            }
        }
        Response::ValidatorPrefsNext(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(prefs) = data.value.prefs {
                tx.send(Action::Validator(
                    ValidatorAction::UpdateValidatorPrefsNext(account_key, prefs),
                ))
                .boxed()?;
            } else {
                warn!("No validator prefs data found for {}", account_key,);
            }
        }
        Response::ValidatorPayee(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::UpdatePayee(
                account_key,
                data.value.payee,
            )))
            .boxed()?;
        }
        Response::Identity(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            if let Some(identity) = data.value.identity {
                tx.send(Action::Validator(ValidatorAction::UpdateIdentity(
                    account_key,
                    identity,
                )))
                .boxed()?;
            } else {
                warn!("No identity data found for {}", account_key,);
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
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::AddAmountToStakeLedger(
                account_key,
                data.value.amount,
            )))
            .boxed()?;
        }
        Response::EventUnbonded(data) => {
            let rc_runtime = runtime.relay_chain();
            let account_key = AccountKey::from_bytes(rc_runtime, data.value.account);
            tx.send(Action::Validator(ValidatorAction::SubChunkFromStakeLedger(
                account_key,
                data.value.chunk,
            )))
            .boxed()?;
        } // _ => {
          //     error!("Unhandled response type: {:?}", response);
          // }
    }
    Ok(())
}

fn spawn_process_transaction_progress(
    runtime: SupportedRuntime,
    progress: TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) {
    let mut progress = progress;
    let tx = tx.clone();
    tokio::spawn(async move {
        if let Err(e) = process_transaction_progress(runtime, &mut progress, &tx).await {
            let _ = tx.send(Action::System(SystemAction::Error(format!(
                "Dispatch error: {}",
                e
            ))));
        }
    });
}

async fn process_transaction_progress(
    runtime: SupportedRuntime,
    progress: &mut TxProgress<SubstrateConfig, OnlineClient<SubstrateConfig>>,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    while let Some(status) = progress.next().await {
        match status.boxed()? {
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
                        let processed_events = process_extrinsic_events(events, runtime);

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
    runtime: SupportedRuntime,
) -> Vec<Response> {
    match runtime {
        SupportedRuntime::Polkadot => suno_polkadot::handle_extrinsic_events(events)
            .unwrap_or_else(|e| {
                error!("Error processing Polkadot extrinsic events: {}", e);
                vec![]
            }),
        SupportedRuntime::Kusama => {
            suno_kusama::handle_extrinsic_events(events).unwrap_or_else(|e| {
                error!("Error processing Kusama extrinsic events: {}", e);
                vec![]
            })
        }
        SupportedRuntime::Paseo => {
            suno_paseo::handle_extrinsic_events(events).unwrap_or_else(|e| {
                error!("Error processing Paseo extrinsic events: {}", e);
                vec![]
            })
        }
        SupportedRuntime::Westend => {
            suno_westend::handle_extrinsic_events(events).unwrap_or_else(|e| {
                error!("Error processing Westend extrinsic events: {}", e);
                vec![]
            })
        }
        SupportedRuntime::AssetHubPolkadot => {
            suno_asset_hub_polkadot::handle_extrinsic_events(events).unwrap_or_else(|e| {
                error!("Error processing AssetHubPolkadot extrinsic events: {}", e);
                vec![]
            })
        }
        SupportedRuntime::AssetHubKusama => suno_asset_hub_kusama::handle_extrinsic_events(events)
            .unwrap_or_else(|e| {
                error!("Error processing AssetHubKusama extrinsic events: {}", e);
                vec![]
            }),
        SupportedRuntime::AssetHubPaseo => suno_asset_hub_paseo::handle_extrinsic_events(events)
            .unwrap_or_else(|e| {
                error!("Error processing AssetHubPaseo extrinsic events: {}", e);
                vec![]
            }),
        SupportedRuntime::AssetHubWestend => {
            suno_asset_hub_westend::handle_extrinsic_events(events).unwrap_or_else(|e| {
                error!("Error processing AssetHubWestend extrinsic events: {}", e);
                vec![]
            })
        }
        _ => {
            vec![]
        }
    }
}
