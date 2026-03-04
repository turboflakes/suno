use crate::call::Call;
use async_trait::async_trait;
use subxt::{utils::AccountId32, OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::Keypair;
use suno_config::Runtime;
use suno_error::{Error, ResultExt};
use suno_primitives::{tx::payload_from_bytes, Response};

#[async_trait]
pub trait RuntimeCaller {
    fn build_call_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        stash: &AccountId32,
        call: Call,
    ) -> Result<Vec<u8>, Error>;

    async fn sign_and_submit_call_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        proxy_signer: &Keypair,
        call_data: Vec<u8>,
    ) -> Result<Response, Error>;
}

#[async_trait]
impl RuntimeCaller for Runtime {
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
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::BondExtra { amount } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Unbond { amount } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_unbond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Rebond { amount } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_rebond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::WithdrawUnbonded => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::Paseo => match call {
                Call::SetSessionKeys { keys } => {
                    let rc = suno_paseo::extrinsics::session_set_keys(keys);
                    suno_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn sign_and_submit_call_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        proxy_signer: &Keypair,
        call_data: Vec<u8>,
    ) -> Result<Response, Error> {
        let payload = payload_from_bytes(call_data);

        let response = api
            .tx()
            .sign_and_submit_then_watch_default(&payload, proxy_signer)
            .await
            .boxed()?;

        Ok(Response::transaction_submitted(response))
    }
}
