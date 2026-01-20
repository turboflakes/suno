use async_trait::async_trait;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use suno_config::Runtime;
use suno_error::Error;
use suno_primitives::{AccountKey, Response};

#[async_trait]
pub trait RuntimeCaller {
    async fn remark_with_event(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: AccountId32,
    ) -> Result<Response, Error>;

    async fn staking_chill(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: AccountId32,
    ) -> Result<Response, Error>;
}

#[async_trait]
impl RuntimeCaller for Runtime {
    async fn remark_with_event(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: AccountId32,
    ) -> Result<Response, Error> {
        match &self {
            Runtime::AssetHubPaseo => {
                let xt = suno_asset_hub_paseo::extrinsics::remark_with_event("some_test".into());
                suno_asset_hub_paseo::submit_as_proxy(&api, xt, stash, None).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    async fn staking_chill(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: AccountId32,
    ) -> Result<Response, Error> {
        match &self {
            Runtime::AssetHubPaseo => {
                let xt = suno_asset_hub_paseo::extrinsics::chill();
                suno_asset_hub_paseo::submit_as_proxy(&api, xt, stash, None).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }
}
