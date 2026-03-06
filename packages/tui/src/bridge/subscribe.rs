use crate::bridge::sync::{spawn_process_block_extrinsics, spawn_process_runtime_events};
use crate::widgets::chains::Chain;
use log::error;
use suno_actions::{network::ConnectionState, Action, ChainAction};
use tokio::sync::mpsc::UnboundedSender;

/// Background task that subscribes head block and sends response over channel.
pub fn subscribe_best_block(chain: &Chain, tx: UnboundedSender<Action>) {
    let api = chain.client().clone();
    let runtime = chain.runtime();
    tokio::spawn(async move {
        match api.stream_best_blocks().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::Chain(ChainAction::UpdateBestBlock(
                                runtime,
                                block.number(),
                            )));
                        }
                        Err(e) => {
                            // TODO: handle disconnection
                            // if e.is_disconnected_will_reconnect() {
                            //     warn!("Lost connection to {} reconnecting...", runtime);
                            //     let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                            //         runtime,
                            //         ConnectionState::Reconnecting,
                            //     )));
                            //     continue;
                            // }
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
    let runtime = chain.runtime();
    tokio::spawn(async move {
        match api.stream_blocks().await {
            Ok(mut blocks_sub) => {
                while let Some(result) = blocks_sub.next().await {
                    match result {
                        Ok(block) => {
                            let _ = tx.send(Action::Chain(ChainAction::UpdateFinalizedBlock(
                                runtime,
                                block.number(),
                                block.hash(),
                            )));

                            // Everytime a new block is received, update the connection state to connected.
                            // Used as KEEPALIVE in case of reconnections and initialization
                            let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                                runtime,
                                ConnectionState::Connected,
                            )));

                            let at_block = match block.at().await {
                                Ok(at_block) => at_block,
                                Err(e) => {
                                    error!("Failed to instantiate a client at this block: {}", e);
                                    continue; // Skip this block and continue with the next one
                                }
                            };

                            // Fetch block events
                            let events = match at_block.events().fetch().await {
                                Ok(events) => events,
                                Err(e) => {
                                    error!("Failed to fetch block events: {}", e);
                                    continue; // Skip this block and continue with the next one
                                }
                            };

                            // Process block events in a separate task
                            spawn_process_runtime_events(&api, block.hash(), events, runtime, &tx);

                            // Fetch block extrinsics
                            let extrinsics = match at_block.extrinsics().fetch().await {
                                Ok(events) => events,
                                Err(e) => {
                                    error!("Failed to fetch block extrinsics: {}", e);
                                    continue; // Skip this block and continue with the next one
                                }
                            };

                            // Process block extrinsics in a separate task
                            spawn_process_block_extrinsics(
                                &api,
                                block.hash(),
                                extrinsics,
                                runtime,
                                &tx,
                            );
                        }
                        Err(e) => {
                            // TODO: handle disconnection
                            // if e.is_disconnected_will_reconnect() {
                            //     info!("Lost connection to {} reconnecting...", runtime.clone());
                            //     let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                            //         runtime,
                            //         ConnectionState::Reconnecting,
                            //     )));
                            //     continue;
                            // }
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
