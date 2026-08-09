use async_trait::async_trait;
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::{ExtrinsicEvents, Extrinsics},
    utils::H256,
    OnlineClient,
};
use suno_config::{CustomConfig, Runtime};
use suno_error::Error;
use suno_primitives::Response;

#[async_trait]
pub trait RuntimeProcessor {
    fn process_transaction_events(
        &self,
        events: ExtrinsicEvents<CustomConfig>,
    ) -> Result<Vec<Response>, Error>;

    async fn process_runtime_events(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        events: Events<CustomConfig>,
    ) -> Result<Vec<Response>, Error>;

    async fn process_block_extrinsics(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        extrinsics: Extrinsics<'_, CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    ) -> Result<Vec<Response>, Error>;
}

#[async_trait]
impl RuntimeProcessor for Runtime {
    fn process_transaction_events(
        &self,
        events: ExtrinsicEvents<CustomConfig>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => suno_polkadot::process_transaction_events(events),
            #[cfg(feature = "kusama")]
            Runtime::Kusama => suno_kusama::process_transaction_events(events),
            #[cfg(feature = "paseo")]
            Runtime::Paseo => suno_paseo::process_transaction_events(events),
            #[cfg(feature = "westend")]
            Runtime::Westend => suno_westend::process_transaction_events(events),
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_transaction_events(events)
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => suno_asset_hub_kusama::process_transaction_events(events),
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::process_transaction_events(events),
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => suno_asset_hub_westend::process_transaction_events(events),
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn process_runtime_events(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        events: Events<CustomConfig>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => {
                suno_polkadot::process_runtime_events(api, block_hash, events).await
            }
            #[cfg(feature = "kusama")]
            Runtime::Kusama => suno_kusama::process_runtime_events(api, block_hash, events).await,
            #[cfg(feature = "paseo")]
            Runtime::Paseo => suno_paseo::process_runtime_events(api, block_hash, events).await,
            #[cfg(feature = "westend")]
            Runtime::Westend => suno_westend::process_runtime_events(api, block_hash, events).await,
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_runtime_events(api, block_hash, events).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::process_runtime_events(api, block_hash, events).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::process_runtime_events(api, block_hash, events).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::process_runtime_events(api, block_hash, events).await
            }
            _ => Ok(vec![]),
        }
    }

    async fn process_block_extrinsics(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        extrinsics: Extrinsics<'_, CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => {
                suno_polkadot::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            #[cfg(feature = "kusama")]
            Runtime::Kusama => {
                suno_kusama::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            #[cfg(feature = "paseo")]
            Runtime::Paseo => {
                suno_paseo::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            #[cfg(feature = "westend")]
            Runtime::Westend => {
                suno_westend::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::process_block_extrinsics(api, block_hash, extrinsics).await
            }
            _ => Ok(vec![]),
        }
    }
}
