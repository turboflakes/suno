use crate::call::Call;
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

    fn build_call_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: &AccountId32,
        call: Call,
    ) -> Result<Vec<u8>, Error>;

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

    fn build_call_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: &AccountId32,
        call: Call,
    ) -> Result<Vec<u8>, Error> {
        match &self {
            Runtime::AssetHubPaseo => match call {
                Call::Bond { amount, payee } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_paseo::wrap_call_into_proxy(&api, rc, stash)
                }
                Call::BondExtra { amount } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(&api, rc, stash)
                }
                Call::Unbond { amount } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_unbond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(&api, rc, stash)
                }
                Call::Rebond { amount } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_rebond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(&api, rc, stash)
                }
                Call::WithdrawUnbonded => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_paseo::wrap_call_into_proxy(&api, rc, stash)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_paseo::wrap_call_into_proxy(&api, rc, stash)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_paseo::wrap_call_into_proxy(&api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
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
