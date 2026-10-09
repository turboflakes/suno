use async_trait::async_trait;
use subxt::{
    client::OnlineClientAtBlockImpl,
    events::Events,
    extrinsics::{ExtrinsicEvents, Extrinsics},
    OnlineClientAtBlock,
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
        api: &OnlineClientAtBlock<CustomConfig>,
        events: Events<CustomConfig>,
    ) -> Result<Vec<Response>, Error>;

    async fn process_block_extrinsics(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
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
            Runtime::CoretimeKusama => suno_coretime_kusama::process_transaction_events(events),
            Runtime::CoretimePolkadot => suno_coretime_polkadot::process_transaction_events(events),
            Runtime::BridgeHubKusama => suno_bridge_hub_kusama::process_transaction_events(events),
            Runtime::BridgeHubPolkadot => {
                suno_bridge_hub_polkadot::process_transaction_events(events)
            }
            Runtime::CollectivesPolkadot => {
                suno_collectives_polkadot::process_transaction_events(events)
            }
            Runtime::CollectivesWestend => {
                suno_collectives_westend::process_transaction_events(events)
            }
            Runtime::PeopleKusama => suno_people_kusama::process_transaction_events(events),
            Runtime::PeoplePolkadot => suno_people_polkadot::process_transaction_events(events),
            Runtime::PeoplePaseo => suno_people_paseo::process_transaction_events(events),
            Runtime::PeopleWestend => suno_people_westend::process_transaction_events(events),
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn process_runtime_events(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        events: Events<CustomConfig>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            Runtime::Polkadot => suno_polkadot::process_runtime_events(api, events).await,
            Runtime::Kusama => suno_kusama::process_runtime_events(api, events).await,
            Runtime::Paseo => suno_paseo::process_runtime_events(api, events).await,
            Runtime::Westend => suno_westend::process_runtime_events(api, events).await,
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_runtime_events(api, events).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::process_runtime_events(api, events).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::process_runtime_events(api, events).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::process_runtime_events(api, events).await
            }
            Runtime::PeoplePolkadot => {
                suno_people_polkadot::process_runtime_events(api, events).await
            }
            Runtime::PeopleKusama => suno_people_kusama::process_runtime_events(api, events).await,
            Runtime::PeoplePaseo => suno_people_paseo::process_runtime_events(api, events).await,
            Runtime::PeopleWestend => {
                suno_people_westend::process_runtime_events(api, events).await
            }
            Runtime::BridgeHubPolkadot => {
                suno_bridge_hub_polkadot::process_runtime_events(api, events).await
            }
            Runtime::BridgeHubKusama => {
                suno_bridge_hub_kusama::process_runtime_events(api, events).await
            }
            Runtime::CoretimePolkadot => {
                suno_coretime_polkadot::process_runtime_events(api, events).await
            }
            Runtime::CoretimeKusama => {
                suno_coretime_kusama::process_runtime_events(api, events).await
            }
            Runtime::CollectivesPolkadot => {
                suno_collectives_polkadot::process_runtime_events(api, events).await
            }
            Runtime::CollectivesWestend => {
                suno_collectives_westend::process_runtime_events(api, events).await
            }
            Runtime::BulletinPolkadot => {
                suno_bulletin_polkadot::process_runtime_events(api, events).await
            }
            Runtime::BulletinPaseo => {
                suno_bulletin_paseo::process_runtime_events(api, events).await
            }

            _ => Ok(vec![]),
        }
    }

    async fn process_block_extrinsics(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        extrinsics: Extrinsics<'_, CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    ) -> Result<Vec<Response>, Error> {
        match &self {
            Runtime::Polkadot => suno_polkadot::process_block_extrinsics(api, extrinsics).await,
            Runtime::Kusama => suno_kusama::process_block_extrinsics(api, extrinsics).await,
            Runtime::Paseo => suno_paseo::process_block_extrinsics(api, extrinsics).await,
            Runtime::Westend => suno_westend::process_block_extrinsics(api, extrinsics).await,
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::CoretimeKusama => {
                suno_coretime_kusama::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::CoretimePolkadot => {
                suno_coretime_polkadot::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::BridgeHubKusama => {
                suno_bridge_hub_kusama::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::BridgeHubPolkadot => {
                suno_bridge_hub_polkadot::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::CollectivesPolkadot => {
                suno_collectives_polkadot::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::CollectivesWestend => {
                suno_collectives_westend::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::PeopleKusama => {
                suno_people_kusama::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::PeoplePolkadot => {
                suno_people_polkadot::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::PeoplePaseo => {
                suno_people_paseo::process_block_extrinsics(api, extrinsics).await
            }
            Runtime::PeopleWestend => {
                suno_people_westend::process_block_extrinsics(api, extrinsics).await
            }
            _ => Ok(vec![]),
        }
    }
}
