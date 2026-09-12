use async_trait::async_trait;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::AccountId32,
    OnlineClientAtBlock,
};
use suno_config::{CustomConfig, Runtime};
use suno_error::Error;
use suno_primitives::{AccountKey, Response};

#[async_trait]
pub trait RuntimeFetcher {
    async fn fetch_era_data(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error>;

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error>;

    async fn fetch_total_staked(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
    ) -> Result<Response, Error>;

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error>;

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error>;

    async fn fetch_validators_era_points(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_validator_points(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validators_authority_status(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_validators_queued_keys(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_validator_next_keys(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_stake_overview(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_stake_ledger(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_prefs(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_prefs_next(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_payee(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_validator_identity(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error>;

    async fn fetch_and_validate_proxy_account(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
        proxy: &AccountId32,
    ) -> Result<Vec<Response>, Error>;

    async fn fetch_account_balance(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
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
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error> {
        match &self {
            Runtime::AssetHubPolkadot => suno_asset_hub_polkadot::fetch_era_data(api).await,
            Runtime::AssetHubKusama => suno_asset_hub_kusama::fetch_era_data(api).await,
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_era_data(api).await,
            Runtime::AssetHubWestend => suno_asset_hub_westend::fetch_era_data(api).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_epoch_data(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error> {
        match self {
            Runtime::Polkadot => suno_polkadot::fetch_epoch_data(api).await,
            Runtime::Kusama => suno_kusama::fetch_epoch_data(api).await,
            Runtime::Paseo => suno_paseo::fetch_epoch_data(api).await,
            Runtime::Westend => suno_westend::fetch_epoch_data(api).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_total_staked(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_staked(api, era_index).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_staked(api, era_index).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_total_staked(api, era_index).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_staked(api, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_active_validators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_validators_count(api, era_index).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_validators_count(api, era_index).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_validators_count(api, era_index).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_validators_count(api, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_active_nominators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_active_nominators_count(api, era_index).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_active_nominators_count(api, era_index).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_active_nominators_count(api, era_index).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_active_nominators_count(api, era_index).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_total_validators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_validators_count(api).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_validators_count(api).await
            }
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_total_validators_count(api).await,
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_validators_count(api).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_total_nominators_count(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_total_nominators_count(api).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_total_nominators_count(api).await
            }
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_total_nominators_count(api).await,
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_total_nominators_count(api).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validators_era_points(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validators_era_points(api, era_index, validator_keys)
                    .await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validators_era_points(api, era_index, validator_keys)
                    .await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validators_era_points(api, era_index, validator_keys)
                    .await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validators_era_points(api, era_index, validator_keys)
                    .await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_points(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::Polkadot => suno_polkadot::fetch_validator_points(api, stash).await,
            Runtime::Kusama => suno_kusama::fetch_validator_points(api, stash).await,
            Runtime::Paseo => suno_paseo::fetch_validator_points(api, stash).await,
            Runtime::Westend => suno_westend::fetch_validator_points(api, stash).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validators_authority_status(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error> {
        match self {
            Runtime::Polkadot => {
                suno_polkadot::fetch_validators_authority_status(api, validator_keys).await
            }
            Runtime::Kusama => {
                suno_kusama::fetch_validators_authority_status(api, validator_keys).await
            }
            Runtime::Paseo => {
                suno_paseo::fetch_validators_authority_status(api, validator_keys).await
            }
            Runtime::Westend => {
                suno_westend::fetch_validators_authority_status(api, validator_keys).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validators_queued_keys(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        validator_keys: &[AccountKey],
    ) -> Result<Vec<Response>, Error> {
        match self {
            Runtime::Polkadot => {
                suno_polkadot::fetch_validators_queued_keys(api, validator_keys).await
            }
            Runtime::Kusama => suno_kusama::fetch_validators_queued_keys(api, validator_keys).await,
            Runtime::Paseo => suno_paseo::fetch_validators_queued_keys(api, validator_keys).await,
            Runtime::Westend => {
                suno_westend::fetch_validators_queued_keys(api, validator_keys).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_next_keys(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::Polkadot => suno_polkadot::fetch_validator_next_keys(api, stash).await,
            Runtime::Kusama => suno_kusama::fetch_validator_next_keys(api, stash).await,
            Runtime::Paseo => suno_paseo::fetch_validator_next_keys(api, stash).await,
            Runtime::Westend => suno_westend::fetch_validator_next_keys(api, stash).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_stake_overview(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_stake_overview(api, era_index, stash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_stake_overview(api, era_index, stash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_stake_overview(api, era_index, stash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_stake_overview(api, era_index, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_stake_ledger(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_staking_ledger(api, stash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_staking_ledger(api, stash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_staking_ledger(api, stash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_staking_ledger(api, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_prefs(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        era_index: u32,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_prefs(api, era_index, stash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_prefs(api, era_index, stash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_prefs(api, era_index, stash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_prefs(api, era_index, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_prefs_next(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_prefs_next(api, stash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_prefs_next(api, stash).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_validator_prefs_next(api, stash).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_prefs_next(api, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_payee(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_validator_payee(api, stash).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_validator_payee(api, stash).await
            }
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_validator_payee(api, stash).await,
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_validator_payee(api, stash).await
            }
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_validator_identity(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::PeoplePolkadot => suno_people_polkadot::fetch_identity(api, stash).await,
            Runtime::PeopleKusama => suno_people_kusama::fetch_identity(api, stash).await,
            Runtime::PeoplePaseo => suno_people_paseo::fetch_identity(api, stash).await,
            Runtime::PeopleWestend => suno_people_westend::fetch_identity(api, stash).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_and_validate_proxy_account(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
        proxy: &AccountId32,
    ) -> Result<Vec<Response>, Error> {
        match self {
            Runtime::AssetHubPolkadot => {
                suno_asset_hub_polkadot::fetch_and_validate_proxy_account(api, stash, proxy).await
            }
            Runtime::AssetHubKusama => {
                suno_asset_hub_kusama::fetch_and_validate_proxy_account(api, stash, proxy).await
            }
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::fetch_and_validate_proxy_account(api, stash, proxy).await
            }
            Runtime::AssetHubWestend => {
                suno_asset_hub_westend::fetch_and_validate_proxy_account(api, stash, proxy).await
            }

            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_account_balance(
        &self,
        api: &OnlineClientAtBlock<CustomConfig>,
        stash: &AccountId32,
    ) -> Result<Response, Error> {
        match self {
            Runtime::AssetHubPolkadot => suno_asset_hub_polkadot::fetch_balance(api, stash).await,
            Runtime::AssetHubKusama => suno_asset_hub_kusama::fetch_balance(api, stash).await,
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_balance(api, stash).await,
            Runtime::AssetHubWestend => suno_asset_hub_westend::fetch_balance(api, stash).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn fetch_metadata(
        &self,
        api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    ) -> Result<Vec<u8>, Error> {
        match self {
            Runtime::AssetHubPolkadot => suno_asset_hub_polkadot::fetch_metadata(api).await,
            Runtime::AssetHubKusama => suno_asset_hub_kusama::fetch_metadata(api).await,
            Runtime::AssetHubPaseo => suno_asset_hub_paseo::fetch_metadata(api).await,
            Runtime::AssetHubWestend => suno_asset_hub_westend::fetch_metadata(api).await,
            Runtime::Polkadot => suno_polkadot::fetch_metadata(api).await,
            Runtime::Kusama => suno_kusama::fetch_metadata(api).await,
            Runtime::Paseo => suno_paseo::fetch_metadata(api).await,
            Runtime::Westend => suno_westend::fetch_metadata(api).await,
            Runtime::PeoplePolkadot => suno_people_polkadot::fetch_metadata(api).await,
            Runtime::PeopleKusama => suno_people_kusama::fetch_metadata(api).await,
            Runtime::PeoplePaseo => suno_people_paseo::fetch_metadata(api).await,
            Runtime::PeopleWestend => suno_people_westend::fetch_metadata(api).await,
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }
}
