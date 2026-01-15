use async_trait::async_trait;
use subxt::{utils::H256, OnlineClient, SubstrateConfig};
use suno_config::SupportedRuntime;
use suno_error::Error;
use suno_primitives::Response;

#[async_trait]
pub trait RuntimeFetcher {
    async fn fetch_era_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_total_staked(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
        era: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
        era: u32,
    ) -> Result<Response, Error>;

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error>;
}

#[async_trait]
impl RuntimeFetcher for SupportedRuntime {
    async fn fetch_era_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error> {
        match self {
            SupportedRuntime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_era_data(api, hash).await
            }
            SupportedRuntime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_era_data(api, hash).await
            }
            SupportedRuntime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_era_data(api, hash).await
            }
            SupportedRuntime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_era_data(api, hash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error> {
        match self {
            SupportedRuntime::Polkadot => suno_polkadot::fetch_epoch_data(api, hash).await,
            SupportedRuntime::Kusama => suno_kusama::fetch_epoch_data(api, hash).await,
            SupportedRuntime::Paseo => suno_paseo::fetch_epoch_data(api, hash).await,
            SupportedRuntime::Westend => suno_westend::fetch_epoch_data(api, hash).await,
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_total_staked(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            SupportedRuntime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_staked(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_staked(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_staked(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_staked(api, hash, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            SupportedRuntime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_validators_count(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_validators_count(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_validators_count(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_validators_count(api, hash, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            SupportedRuntime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_nominators_count(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_nominators_count(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_nominators_count(api, hash, era_index).await
            }
            SupportedRuntime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_nominators_count(api, hash, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error> {
        match self {
            SupportedRuntime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_validators_count(api, hash).await
            }
            SupportedRuntime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_validators_count(api, hash).await
            }
            SupportedRuntime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_validators_count(api, hash).await
            }
            SupportedRuntime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_validators_count(api, hash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        hash: H256,
    ) -> Result<Response, Error> {
        match self {
            SupportedRuntime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_nominators_count(api, hash).await
            }
            SupportedRuntime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_nominators_count(api, hash).await
            }
            SupportedRuntime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_nominators_count(api, hash).await
            }
            SupportedRuntime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_nominators_count(api, hash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }
}
