use crate::bridge::dispatch::dispatch_response_action;
use crate::widgets::chains::Chain;
use log::{error, info, warn};

use subxt::{events::Events, utils::H256, OnlineClient, SubstrateConfig};
use suno_actions::{network::ConnectionState, Action, ChainAction, SystemAction};
use suno_config::SupportedRuntime;
use suno_error::Error;
use suno_primitives::Response;
use tokio::sync::mpsc::UnboundedSender;

/// Background task that subscribes head block and sends response over channel.
pub fn subscribe_best_block(chain: &Chain, tx: UnboundedSender<Action>) {
    let api = chain.client().clone();
    let runtime = chain.runtime().clone();
    tokio::spawn(async move {
        match api.blocks().subscribe_best().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::Chain(ChainAction::UpdateBestBlock(
                                runtime.clone(),
                                block.number().into(),
                            )));
                        }
                        Err(e) => {
                            if e.is_disconnected_will_reconnect() {
                                warn!("Lost connection to {} reconnecting...", runtime.clone());
                                let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                                    runtime.clone(),
                                    ConnectionState::Reconnecting,
                                )));
                                continue;
                            }
                            error!("subscribe_best result error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("subscribe_best error: {:?}", e);
            }
        }
    });
}

/// Background task that subscribes finalized block and sends response over channel.
pub fn subscribe_finalized_block(chain: &Chain, tx: UnboundedSender<Action>) {
    let api = chain.client().clone();
    let runtime = chain.runtime().clone();
    tokio::spawn(async move {
        match api.blocks().subscribe_finalized().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::Chain(ChainAction::UpdateFinalizedBlock(
                                runtime.clone(),
                                block.number().into(),
                                block.hash(),
                            )));

                            // Everytime a new block is received, update the connection state to connected.
                            // Used as KEEPALIVE in case of reconnections and initialization
                            let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                                runtime.clone(),
                                ConnectionState::Connected,
                            )));

                            // Fetch block events
                            let events = match block.events().await {
                                Ok(events) => events,
                                Err(e) => {
                                    error!("Failed to fetch block events: {}", e);
                                    continue; // Skip this block and continue with the next one
                                }
                            };

                            // Process block events in a separate task
                            spawn_process_runtime_events(&api, block.hash(), events, &runtime, &tx);
                        }
                        Err(e) => {
                            if e.is_disconnected_will_reconnect() {
                                info!("Lost connection to {} reconnecting...", runtime.clone());
                                let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                                    runtime.clone(),
                                    ConnectionState::Reconnecting,
                                )));
                                continue;
                            }
                            error!("subscribe_finalized result error: {}", e);
                        }
                    }
                }
            }
            Err(e) => {
                error!("subscribe_finalized error: {:?}", e);
            }
        }
    });
}

pub fn spawn_process_runtime_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) {
    let api = api.clone();
    let runtime = runtime.clone();
    let tx = tx.clone();

    tokio::spawn(async move {
        if let Err(e) = process_runtime_events(&api, block_hash, events, &runtime, &tx).await {
            let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
        }
    });
}

async fn process_runtime_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
    runtime: &SupportedRuntime,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    let processed_events = handle_runtime_events(api, block_hash, events, runtime).await;

    for response in processed_events {
        dispatch_response_action(response, runtime, tx)?;
    }

    Ok(())
}

async fn handle_runtime_events(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    events: Events<SubstrateConfig>,
    runtime: &SupportedRuntime,
) -> Vec<Response> {
    match runtime {
        SupportedRuntime::Polkadot => suno_polkadot::handle_events(api, block_hash, events)
            .await
            .unwrap_or_else(|e| {
                error!("Error processing Polkadot events: {}", e);
                vec![]
            }),
        SupportedRuntime::Kusama => suno_kusama::handle_events(api, block_hash, events)
            .await
            .unwrap_or_else(|e| {
                error!("Error processing Kusama events: {}", e);
                vec![]
            }),
        SupportedRuntime::Paseo => suno_paseo::handle_events(api, block_hash, events)
            .await
            .unwrap_or_else(|e| {
                error!("Error processing Paseo events: {}", e);
                vec![]
            }),
        SupportedRuntime::Westend => suno_westend::handle_events(api, block_hash, events)
            .await
            .unwrap_or_else(|e| {
                error!("Error processing Westend events: {}", e);
                vec![]
            }),
        SupportedRuntime::AssetHubPolkadot => {
            suno_asset_hub_polkadot::handle_events(api, block_hash, events)
                .await
                .unwrap_or_else(|e| {
                    error!("Error processing AssetHubPolkadot events: {}", e);
                    vec![]
                })
        }
        SupportedRuntime::AssetHubKusama => {
            suno_asset_hub_kusama::handle_events(api, block_hash, events)
                .await
                .unwrap_or_else(|e| {
                    error!("Error processing AssetHubKusama events: {}", e);
                    vec![]
                })
        }
        SupportedRuntime::AssetHubPaseo => {
            suno_asset_hub_paseo::handle_events(api, block_hash, events)
                .await
                .unwrap_or_else(|e| {
                    error!("Error processing AssetHubPaseo events: {}", e);
                    vec![]
                })
        }
        SupportedRuntime::AssetHubWestend => {
            suno_asset_hub_westend::handle_events(api, block_hash, events)
                .await
                .unwrap_or_else(|e| {
                    error!("Error processing AssetHubWestend events: {}", e);
                    vec![]
                })
        }
        _ => {
            vec![]
        }
    }
}
