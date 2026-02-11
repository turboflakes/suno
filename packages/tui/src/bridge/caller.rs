use async_trait::async_trait;
use subxt::{
    utils::{AccountId32, H256},
    OnlineClient, SubstrateConfig,
};
use subxt_signer::sr25519::Keypair;
use suno_config::Runtime;
use suno_error::Error;
use suno_primitives::{AccountKey, Response};

#[async_trait]
pub trait RuntimeCaller {
    async fn sign_and_submit(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        signer: &Keypair,
        call_data: Vec<u8>,
    ) -> Result<Response, Error>;

    fn remark_with_event(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: &AccountId32,
    ) -> Result<Vec<u8>, Error>;

    // async fn staking_chill(
    //     &self,
    //     api: &OnlineClient<SubstrateConfig>,
    //     stash: &AccountId32,
    //     signer: &Keypair,
    // ) -> Result<Response, Error>;
}

#[async_trait]
impl RuntimeCaller for Runtime {
    async fn sign_and_submit(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        signer: &Keypair,
        call_data: Vec<u8>,
    ) -> Result<Response, Error> {
        match &self {
            Runtime::AssetHubPaseo => {
                suno_asset_hub_paseo::sign_and_submit_call_data(&api, signer, call_data).await
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    fn remark_with_event(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: &AccountId32,
    ) -> Result<Vec<u8>, Error> {
        match &self {
            Runtime::AssetHubPaseo => {
                let call = suno_asset_hub_paseo::extrinsics::remark_with_event("some_test".into());
                suno_asset_hub_paseo::wrap_call_into_proxy(&api, call, stash)
            }
            _ => Err(Error::UnsupportedRuntime(self.clone())),
        }
    }

    // async fn staking_chill(
    //     &self,
    //     api: &OnlineClient<SubstrateConfig>,
    //     stash: &AccountId32,
    //     signer: &Keypair,
    // ) -> Result<Response, Error> {
    //     match &self {
    //         Runtime::AssetHubPaseo => {
    //             let call = suno_asset_hub_paseo::extrinsics::chill();
    //             suno_asset_hub_paseo::submit_as_proxy(&api, call, stash, signer).await
    //         }
    //         _ => Err(Error::UnsupportedRuntime(self.clone())),
    //     }
    // }
}
