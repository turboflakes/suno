use crate::bridge::sync::{spawn_process_block_extrinsics, spawn_process_runtime_events};
use std::{fmt::Display, future::Future, time::Duration};
use suno_actions::{Action, ChainAction};
use suno_config::SupportedRuntime;
use suno_primitives::{chain::Chain, network::ConnectionState};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::timeout;
use tracing::{error, info};

/// Maximum time to wait to initialize subscription before
/// considering it offline and breaking out of the task.
const SUBSCRIPTION_TIMEOUT: Duration = Duration::from_millis(6_000);

/// Maximum time to wait for the next block stream before
/// considering it stalled and breaking out of the task.
const BLOCK_TIMEOUT: Duration = Duration::from_millis(30_000);

/// Maximum time to wait for network requests before
/// considering it stalled and breaking out of the task.
const NETWORK_TIMEOUT: Duration = Duration::from_millis(10_000);

/// Background task that subscribes head block and sends response over channel.
pub fn subscribe_best_block(chain: &Chain, tx: UnboundedSender<Action>) {
    info!("Subscribing to best blocks on {}", chain.name());
    let api = chain.client().clone();
    let runtime = chain.runtime();

    tokio::spawn(async move {
        let mut blocks_sub = match with_timeout_and_connection_state(
            api.stream_blocks(),
            "Subscription for best block",
            runtime,
            &tx,
            SUBSCRIPTION_TIMEOUT,
            ConnectionState::BestBlockSubcriptionDropped,
        )
        .await
        {
            Some(result) => result,
            None => return,
        };

        loop {
            let result = match timeout(BLOCK_TIMEOUT, blocks_sub.next()).await {
                Ok(Some(result)) => result,
                _ => {
                    error!(
                        "Next best block on {} timed out after {:?}",
                        runtime.to_string(),
                        BLOCK_TIMEOUT
                    );
                    let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                        runtime,
                        ConnectionState::BestBlockSubcriptionDropped,
                    )));
                    break;
                }
            };

            match result {
                Ok(block) => {
                    let _ = tx.send(Action::Chain(ChainAction::UpdateBestBlock(
                        runtime,
                        block.number(),
                    )));
                }
                Err(e) => {
                    error!("{e}");
                    let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                        runtime,
                        ConnectionState::BestBlockSubcriptionDropped,
                    )));
                    break;
                }
            }
        }
    });
}

/// Background task that subscribes finalized block and sends response over channel.
pub fn subscribe_finalized_block(chain: &Chain, tx: UnboundedSender<Action>) {
    info!("Subscribing to finalized blocks on {}", chain.name());
    let api = chain.client().clone();
    let runtime = chain.runtime();
    tokio::spawn(async move {
        let mut blocks_sub = match with_timeout_and_connection_state(
            api.stream_blocks(),
            "Subscription for finalized block",
            runtime,
            &tx,
            SUBSCRIPTION_TIMEOUT,
            ConnectionState::FinalizedSubscriptionDropped,
        )
        .await
        {
            Some(result) => result,
            None => return,
        };

        loop {
            let result = match timeout(BLOCK_TIMEOUT, blocks_sub.next()).await {
                Ok(Some(result)) => result,
                _ => {
                    error!(
                        "Next finalized block on {} timed out after {:?}",
                        runtime.to_string(),
                        BLOCK_TIMEOUT
                    );
                    let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                        runtime,
                        ConnectionState::FinalizedSubscriptionDropped,
                    )));
                    break;
                }
            };

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

                    // Fetch latest block
                    let at_block = match with_timeout_and_connection_state(
                        block.at(),
                        "Fetch latest finalized block",
                        runtime,
                        &tx,
                        NETWORK_TIMEOUT,
                        ConnectionState::FinalizedSubscriptionDropped,
                    )
                    .await
                    {
                        Some(block) => block,
                        None => break, // or return, depending on severity
                    };

                    // Fetch block events
                    let events = match with_timeout_and_connection_state(
                        at_block.events().fetch(),
                        "Fetch events",
                        runtime,
                        &tx,
                        NETWORK_TIMEOUT,
                        ConnectionState::FinalizedSubscriptionDropped,
                    )
                    .await
                    {
                        Some(events) => events,
                        None => break, // or return, depending on severity
                    };

                    // Process block events in a separate task
                    spawn_process_runtime_events(&api, block.hash(), runtime, events, &tx);

                    // Fetch block extrinsics
                    let extrinsics = match with_timeout_and_connection_state(
                        at_block.extrinsics().fetch(),
                        "Fetch extrinsics",
                        runtime,
                        &tx,
                        NETWORK_TIMEOUT,
                        ConnectionState::FinalizedSubscriptionDropped,
                    )
                    .await
                    {
                        Some(extrinsics) => extrinsics,
                        None => break,
                    };

                    // Process block extrinsics in a separate task
                    spawn_process_block_extrinsics(&api, block.hash(), runtime, extrinsics, &tx);
                }
                Err(e) => {
                    error!("{e}");
                    let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                        runtime,
                        ConnectionState::FinalizedSubscriptionDropped,
                    )));
                    break;
                }
            }
        }
    });
}

async fn with_timeout_and_connection_state<F, T, E>(
    operation: F,
    operation_name: &str,
    runtime: SupportedRuntime,
    tx: &UnboundedSender<Action>,
    timeout_duration: Duration,
    connection_state: ConnectionState,
) -> Option<T>
where
    F: Future<Output = Result<T, E>>,
    E: Display,
{
    match timeout(timeout_duration, operation).await {
        Ok(Ok(value)) => Some(value),
        Ok(Err(e)) => {
            error!("Failed to {}: {}", operation_name, e);
            None
        }
        Err(_) => {
            error!(
                "{} on {} timed out after {:?}",
                operation_name,
                runtime.to_string(),
                timeout_duration
            );
            let _ = tx.send(Action::Chain(ChainAction::UpdateConnectionState(
                runtime,
                connection_state,
            )));
            None
        }
    }
}
