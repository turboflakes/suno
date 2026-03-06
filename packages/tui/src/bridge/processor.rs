use async_trait::async_trait;
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::{ExtrinsicEvents, Extrinsics},
    utils::H256,
    OnlineClient, SubstrateConfig,
};
use suno_config::Runtime;
use suno_error::Error;
use suno_primitives::Response;

#[async_trait]
pub trait RuntimeProcessor {
    fn process_transaction_events(
        &self,
        events: ExtrinsicEvents<SubstrateConfig>,
    ) -> Result<Vec<Response>, Error>;

    async fn process_runtime_events(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        events: Events<SubstrateConfig>,
    ) -> Result<Vec<Response>, Error>;

    async fn process_block_extrinsics(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        extrinsics: Extrinsics<'_, SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    ) -> Result<Vec<Response>, Error>;
}

#[async_trait]
impl RuntimeProcessor for Runtime {
    fn process_transaction_events(
        &self,
        events: ExtrinsicEvents<SubstrateConfig>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            Runtime::Polkadot => suno_polkadot::process_transaction_events(events),
            Runtime::Kusama => suno_kusama::process_transaction_events(events),
            Runtime::Paseo => suno_paseo::process_transaction_events(events),
            Runtime::Westend => suno_westend::process_transaction_events(events),
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_transaction_events(events)
            }
            Runtime::AssetHubKusama => suno_asset_hub_kusama::process_transaction_events(events),
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::process_transaction_events(events),
            Runtime::AssetHubWestend => suno_asset_hub_westend::process_transaction_events(events),
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn process_runtime_events(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        events: Events<SubstrateConfig>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            Runtime::Polkadot => {
                suno_polkadot::process_runtime_events(api, block_hash, events).await
            }
            Runtime::Kusama => suno_kusama::process_runtime_events(api, block_hash, events).await,
            Runtime::Paseo => suno_paseo::process_runtime_events(api, block_hash, events).await,
            Runtime::Westend => suno_westend::process_runtime_events(api, block_hash, events).await,
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_runtime_events(api, block_hash, events).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::process_runtime_events(api, block_hash, events).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::process_runtime_events(api, block_hash, events).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::process_runtime_events(api, block_hash, events).await
            }
            _ => Ok(vec![]),
        }
    }

    async fn process_block_extrinsics(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        extrinsics: Extrinsics<'_, SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            Runtime::Polkadot => {
                suno_polkadot::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            Runtime::Kusama => {
                suno_kusama::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            Runtime::Paseo => {
                suno_paseo::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            Runtime::Westend => {
                suno_westend::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            _ => Ok(vec![]),
        }
    }
}
