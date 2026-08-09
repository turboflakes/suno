use async_trait::async_trait;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::{AccountId32, H256},
    OnlineClient,
};
use suno_config::{CustomConfig, Runtime};
use suno_error::Error;
use suno_primitives::{AccountKey, Response};

#[async_trait]
pub trait RuntimeFetcher {
    async fn fetch_era_data(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_total_staked(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error>;

    async fn fetch_validators_era_points(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_validator_points(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validators_authority_status(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_validators_queued_keys(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_validator_next_keys(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_stake_overview(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_stake_ledger(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_prefs(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_prefs_next(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_payee(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_identity(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_and_validate_proxy_account(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
        proxy: &AccountId32,
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_account_balance(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_metadata(
        &self,
        api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    ) -> Result<Vec<u8>, Error>;
}

#[async_trait]
impl RuntimeFetcher for Runtime {
    async fn fetch_era_data(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match &self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_era_data(api, block_hash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => suno_asset_hub_kusama::fetch_era_data(api, block_hash).await,
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_era_data(api, block_hash).await,
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_era_data(api, block_hash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => suno_polkadot::fetch_epoch_data(api, block_hash).await,
            #[cfg(feature = "kusama")]
            Runtime::Kusama => suno_kusama::fetch_epoch_data(api, block_hash).await,
            #[cfg(feature = "paseo")]
            Runtime::Paseo => suno_paseo::fetch_epoch_data(api, block_hash).await,
            #[cfg(feature = "westend")]
            Runtime::Westend => suno_westend::fetch_epoch_data(api, block_hash).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_total_staked(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_staked(api, block_hash, era_index).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_staked(api, block_hash, era_index).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_staked(api, block_hash, era_index).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_staked(api, block_hash, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_validators_count(api, block_hash, era_index)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_nominators_count(api, block_hash, era_index)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_validators_count(api, block_hash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_validators_count(api, block_hash).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_validators_count(api, block_hash).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_validators_count(api, block_hash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_nominators_count(api, block_hash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_nominators_count(api, block_hash).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_nominators_count(api, block_hash).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_nominators_count(api, block_hash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validators_era_points(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validators_era_points(
                    api,
                    block_hash,
                    era_index,
                    validator_keys,
                )
                .await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_points(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => {
                suno_polkadot::fetch_validator_points(api, block_hash, stash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::Kusama => suno_kusama::fetch_validator_points(api, block_hash, stash).await,
            #[cfg(feature = "paseo")]
            Runtime::Paseo => suno_paseo::fetch_validator_points(api, block_hash, stash).await,
            #[cfg(feature = "westend")]
            Runtime::Westend => suno_westend::fetch_validator_points(api, block_hash, stash).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validators_authority_status(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => {
                suno_polkadot::fetch_validators_authority_status(api, block_hash, validator_keys)
                    .await
            }
            #[cfg(feature = "kusama")]
            Runtime::Kusama => {
                suno_kusama::fetch_validators_authority_status(api, block_hash, validator_keys)
                    .await
            }
            #[cfg(feature = "paseo")]
            Runtime::Paseo => {
                suno_paseo::fetch_validators_authority_status(api, block_hash, validator_keys).await
            }
            #[cfg(feature = "westend")]
            Runtime::Westend => {
                suno_westend::fetch_validators_authority_status(api, block_hash, validator_keys)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validators_queued_keys(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => {
                suno_polkadot::fetch_validators_queued_keys(api, block_hash, validator_keys).await
            }
            #[cfg(feature = "kusama")]
            Runtime::Kusama => {
                suno_kusama::fetch_validators_queued_keys(api, block_hash, validator_keys).await
            }
            #[cfg(feature = "paseo")]
            Runtime::Paseo => {
                suno_paseo::fetch_validators_queued_keys(api, block_hash, validator_keys).await
            }
            #[cfg(feature = "westend")]
            Runtime::Westend => {
                suno_westend::fetch_validators_queued_keys(api, block_hash, validator_keys).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_next_keys(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => {
                suno_polkadot::fetch_validator_next_keys(api, block_hash, stash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::Kusama => suno_kusama::fetch_validator_next_keys(api, block_hash, stash).await,
            #[cfg(feature = "paseo")]
            Runtime::Paseo => suno_paseo::fetch_validator_next_keys(api, block_hash, stash).await,
            #[cfg(feature = "westend")]
            Runtime::Westend => {
                suno_westend::fetch_validator_next_keys(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_stake_overview(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_stake_overview(
                    api, block_hash, era_index, stash,
                )
                .await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_stake_ledger(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_staking_ledger(api, block_hash, stash)
                    .await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_staking_ledger(api, block_hash, stash).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_staking_ledger(api, block_hash, stash).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_staking_ledger(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_prefs(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_prefs(api, block_hash, era_index, stash)
                    .await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_prefs(api, block_hash, era_index, stash)
                    .await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_prefs(api, block_hash, era_index, stash).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_prefs(api, block_hash, era_index, stash)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_prefs_next(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_prefs_next(api, block_hash, stash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_prefs_next(api, block_hash, stash).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_prefs_next(api, block_hash, stash).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_prefs_next(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_payee(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_payee(api, block_hash, stash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_payee(api, block_hash, stash).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_payee(api, block_hash, stash).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_payee(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_identity(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::PeoplePolkadot => {
                suno_people_polkadot::fetch_identity(api, block_hash, stash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::PeopleKusama => {
                suno_people_kusama::fetch_identity(api, block_hash, stash).await
            }
            #[cfg(feature = "paseo")]
            Runtime::PeoplePaseo => suno_people_paseo::fetch_identity(api, block_hash, stash).await,
            #[cfg(feature = "westend")]
            Runtime::PeopleWestend => {
                suno_people_westend::fetch_identity(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_and_validate_proxy_account(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
        proxy: &AccountId32,
    ) -> Result<Vec<Response>, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_and_validate_proxy_account(
                    api, block_hash, stash, proxy,
                )
                .await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_and_validate_proxy_account(
                    api, block_hash, stash, proxy,
                )
                .await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_and_validate_proxy_account(
                    api, block_hash, stash, proxy,
                )
                .await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_and_validate_proxy_account(
                    api, block_hash, stash, proxy,
                )
                .await
            }

            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_account_balance(
        &self,
        api: &OnlineClient<CustomConfig>,
        block_hash: H256,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_balance(api, block_hash, stash).await
            }
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_balance(api, block_hash, stash).await
            }
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_balance(api, block_hash, stash).await
            }
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_balance(api, block_hash, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_metadata(
        &self,
        api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    ) -> Result<Vec<u8>, Error> {
        match self {
            #[cfg(feature = "polkadot")]
            Runtime::AssetHubPolkadot => suno_asset_hub_polkadot::fetch_metadata(api).await,
            #[cfg(feature = "kusama")]
            Runtime::AssetHubKusama => suno_asset_hub_kusama::fetch_metadata(api).await,
            #[cfg(feature = "paseo")]
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_metadata(api).await,
            #[cfg(feature = "westend")]
            Runtime::AssetHubWestend => suno_asset_hub_westend::fetch_metadata(api).await,
            #[cfg(feature = "polkadot")]
            Runtime::Polkadot => suno_polkadot::fetch_metadata(api).await,
            #[cfg(feature = "kusama")]
            Runtime::Kusama => suno_kusama::fetch_metadata(api).await,
            #[cfg(feature = "paseo")]
            Runtime::Paseo => suno_paseo::fetch_metadata(api).await,
            #[cfg(feature = "westend")]
            Runtime::Westend => suno_westend::fetch_metadata(api).await,
            #[cfg(feature = "polkadot")]
            Runtime::PeoplePolkadot => suno_people_polkadot::fetch_metadata(api).await,
            #[cfg(feature = "kusama")]
            Runtime::PeopleKusama => suno_people_kusama::fetch_metadata(api).await,
            #[cfg(feature = "paseo")]
            Runtime::PeoplePaseo => suno_people_paseo::fetch_metadata(api).await,
            #[cfg(feature = "westend")]
            Runtime::PeopleWestend => suno_people_westend::fetch_metadata(api).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }
}
