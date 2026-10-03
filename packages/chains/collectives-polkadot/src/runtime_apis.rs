use super::node_runtime;
use subxt::client::{ClientAtBlock, OnlineClientAtBlockImpl};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};

/// Fetch metadata V14 required for Polkadot Vault
pub async fn fetch_metadata(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
) -> Result<Vec<u8>, Error> {
    let addr = node_runtime::runtime_apis()
        .metadata()
        .metadata_at_version(14u32);
    let value = api
        .runtime_apis()
        .call(addr)
        .await
        .boxed()?
        .ok_or_else(|| Error::MetadataV14)?;

    Ok(value.0)
}
