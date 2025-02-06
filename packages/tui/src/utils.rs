use log::{info, warn};
use std::time::Duration;
use subxt::{
    backend::rpc::reconnecting_rpc_client::{ExponentialBackoff, RpcClient},
    utils::validate_url_is_secure,
};

pub async fn create_substrate_rpc_client_from_url(
    url: &str,
) -> Result<RpcClient, Box<dyn std::error::Error>> {
    if let Err(_) = validate_url_is_secure(url) {
        warn!("Insecure URL provided: {}", url);
    };
    info!("Connecting to Rpc endpoint {}", url);
    let rpc = RpcClient::builder()
        .retry_policy(ExponentialBackoff::from_millis(100).max_delay(Duration::from_secs(10)))
        .build(url.to_string())
        .await?;
    Ok(rpc)
}
