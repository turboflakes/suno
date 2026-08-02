use std::time::Duration;
use subxt::{
    ext::jsonrpsee::ws_client::PingConfig,
    lightclient::{LightClient, LightClientError, LightClientRpc},
};
use subxt_rpcs::{
    client::{
        reconnecting_rpc_client::{ExponentialBackoff, RpcClient as ReconnectingRpcClient},
        RpcClient,
    },
    utils::validate_url_is_secure,
};
use suno_config::{ChainConfig, SupportedRuntime};
use tracing::{info, warn};

pub async fn _create_substrate_rpc_client_from_url(
    url: &str,
) -> Result<RpcClient, Box<dyn std::error::Error>> {
    if validate_url_is_secure(url).is_err() {
        warn!("Insecure URL provided: {}", url);
    };
    let rpc = RpcClient::from_insecure_url(url).await?;
    info!("Connected to RPC endpoint {}", url);
    Ok(rpc)
}

pub async fn create_substrate_rpc_client_from_url(
    url: &str,
) -> Result<ReconnectingRpcClient, Box<dyn std::error::Error>> {
    if validate_url_is_secure(url).is_err() {
        warn!("Insecure URL provided: {}", url);
    };
    let ping_config = PingConfig::new();
    ping_config.ping_interval(Duration::from_secs(12));
    ping_config.inactive_limit(Duration::from_secs(18));
    let rpc = ReconnectingRpcClient::builder()
        .retry_policy(ExponentialBackoff::from_millis(10).max_delay(Duration::from_secs(6)))
        .enable_ws_ping(ping_config)
        .request_timeout(Duration::from_secs(24))
        .connection_timeout(Duration::from_secs(6))
        .build(url.to_string())
        .await?;
    info!("Connected to RPC endpoint {}", url);
    Ok(rpc)
}

pub async fn create_rpc_client_from_config(
    runtime: &SupportedRuntime,
    chain_config: &ChainConfig,
    relay_optional: Option<(SupportedRuntime, LightClient)>,
) -> Result<(RpcClient, Option<(SupportedRuntime, LightClient)>), Box<dyn std::error::Error>> {
    match (
        runtime.is_relay_chain(),
        chain_config.rpc_url.trim().is_empty(),
        relay_optional,
    ) {
        // Explicit RPC override always wins (relay chain or parachain).
        (_, false, _) => {
            let rpc = create_substrate_rpc_client_from_url(&chain_config.rpc_url).await?;
            Ok((rpc.into(), None))
        }
        // Relay chain default: create a light client when no rpc_url is configured.
        (true, true, _) => {
            let (lc, rpc) =
                create_light_client_from_relay_chain_specs(runtime.chain_specs()).await?;
            Ok((rpc.into(), Some((*runtime, lc))))
        }
        // Parachain default: derive from the matching relay light client.
        (false, true, Some((relay_runtime, lc))) if relay_runtime == runtime.relay_chain() => {
            let rpc = lc.parachain(runtime.chain_specs())?;
            Ok((rpc.into(), None))
        }
        // Parachain without rpc_url but missing relay light client.
        (false, true, _) => Err(format!(
            "Missing relay chain for {runtime}; Configure its relay chain before this chain, or set `rpc_url`"
        )
        .into()),
    }
}

async fn create_light_client_from_relay_chain_specs(
    chain_specs: &str,
) -> Result<(LightClient, LightClientRpc), LightClientError> {
    let (lc, rpc) = LightClient::relay_chain(chain_specs)?;

    Ok((lc, rpc))
}
