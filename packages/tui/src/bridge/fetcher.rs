use async_trait::async_trait;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_config::Runtime;
use suno_error::Error;
use suno_primitives::{AccountKey, Response};

#[async_trait]
pub trait RuntimeFetcher {
    async fn fetch_era_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_total_staked(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_validators_era_points(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
        validator_keys: &Vec<AccountKey>,
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_validator_points(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validators_authority_status(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        validator_keys: &Vec<AccountKey>,
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_stake_overview(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_stake_ledger(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_commission(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;
}

#[async_trait]
impl RuntimeFetcher for Runtime {
    async fn fetch_era_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match &self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_era_data(api, block_hash).await
            }
            Runtime::AssetHubKusama => suno_asset_hub_kusama::fetch_era_data(api, block_hash).await,
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_era_data(api, block_hash).await,
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_era_data(api, block_hash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match self {
            Runtime::Polkadot => suno_polkadot::fetch_epoch_data(api, block_hash).await,
            Runtime::Kusama => suno_kusama::fetch_epoch_data(api, block_hash).await,
            Runtime::Paseo => suno_paseo::fetch_epoch_data(api, block_hash).await,
            Runtime::Westend => suno_westend::fetch_epoch_data(api, block_hash).await,
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_total_staked(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_staked(api, block_hash, era_index).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_staked(api, block_hash, era_index).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_staked(api, block_hash, era_index).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_staked(api, block_hash, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_validators_count(api, block_hash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_validators_count(api, block_hash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_validators_count(api, block_hash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_validators_count(api, block_hash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_nominators_count(api, block_hash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_nominators_count(api, block_hash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_nominators_count(api, block_hash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_nominators_count(api, block_hash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_validators_era_points(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
        validator_keys: &Vec<AccountKey>,
    ) -> Result<Vec<Response>, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_validator_points(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::Polkadot => {
                suno_polkadot::fetch_validator_points(api, block_hash, stash).await
            }
            Runtime::Kusama => suno_kusama::fetch_validator_points(api, block_hash, stash).await,
            Runtime::Paseo => suno_paseo::fetch_validator_points(api, block_hash, stash).await,
            Runtime::Westend => suno_westend::fetch_validator_points(api, block_hash, stash).await,
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_validators_authority_status(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        validator_keys: &Vec<AccountKey>,
    ) -> Result<Vec<Response>, Error> {
        match self {
            Runtime::Polkadot => {
                suno_polkadot::fetch_validators_authority_status(api, block_hash, validator_keys)
                    .await
            }
            Runtime::Kusama => {
                suno_kusama::fetch_validators_authority_status(api, block_hash, validator_keys)
                    .await
            }
            Runtime::Paseo => {
                suno_paseo::fetch_validators_authority_status(api, block_hash, validator_keys).await
            }
            Runtime::Westend => {
                suno_westend::fetch_validators_authority_status(api, block_hash, validator_keys)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_stake_overview(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_stake_ledger(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_staking_ledger(api, block_hash, stash)
                    .await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_staking_ledger(api, block_hash, stash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_staking_ledger(api, block_hash, stash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_staking_ledger(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn fetch_validator_commission(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_commission(api, block_hash, stash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_commission(api, block_hash, stash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_commission(api, block_hash, stash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_commission(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }
}
