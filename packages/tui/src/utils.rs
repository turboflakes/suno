use log::{info, warn};
use std::time::Duration;
use subxt::{
    backend::rpc::{
        reconnecting_rpc_client::{ExponentialBackoff, RpcClient as ReconnectingRpcClient},
        RpcClient,
    },
    ext::{jsonrpsee::ws_client::PingConfig, subxt_rpcs::utils::validate_url_is_secure},
};

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
