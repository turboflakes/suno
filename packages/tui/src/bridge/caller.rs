use async_trait::async_trait;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::AccountId32,
    OnlineClient, SubstrateConfig,
};
use subxt_signer::sr25519::Keypair;
use suno_config::Runtime;
use suno_error::{Error, ResultExt};
use suno_primitives::{
    call::Call,
    tx::{Bytes, RawPayload},
    Response,
};

#[async_trait]
pub trait RuntimeCaller {
    fn build_call_data(
        &self,
        api: &ClientAtBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
        stash: &AccountId32,
        call: Call,
    ) -> Result<Bytes, Error>;

    async fn sign_and_submit_call_data(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        proxy_signer: &Keypair,
        call_data: &[u8],
    ) -> Result<Response, Error>;
}

#[async_trait]
impl RuntimeCaller for Runtime {
    fn build_call_data(
        &self,
        api: &ClientAtBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
        stash: &AccountId32,
        call: Call,
    ) -> Result<Bytes, Error> {
        match &self {
            Runtime::AssetHubPolkadot => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_unbond(amount);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_rebond(amount);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                Call::WithdrawUnbonded { .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Chill => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_chill();
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::AssetHubKusama => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_unbond(amount);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_rebond(amount);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                Call::WithdrawUnbonded { .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Chill => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_chill();
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::AssetHubPaseo => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_unbond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_rebond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                Call::WithdrawUnbonded { .. } => {
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
                Call::Chill => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_chill();
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::AssetHubWestend => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_unbond(amount);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_rebond(amount);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash)
                }
                Call::WithdrawUnbonded { .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash)
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
                Call::Chill => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_chill();
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::Polkadot => match call {
                Call::SetSessionKeys { keys } => {
                    let rc = suno_polkadot::extrinsics::session_set_keys(keys);
                    suno_polkadot::wrap_call_into_proxy(api, rc, stash)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::Kusama => match call {
                Call::SetSessionKeys { keys } => {
                    let rc = suno_kusama::extrinsics::session_set_keys(keys);
                    suno_kusama::wrap_call_into_proxy(api, rc, stash)
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
            Runtime::Westend => match call {
                Call::SetSessionKeys { keys } => {
                    let rc = suno_westend::extrinsics::session_set_keys(keys);
                    suno_westend::wrap_call_into_proxy(api, rc, stash)
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
        call_data: &[u8],
    ) -> Result<Response, Error> {
        let at_block = api.at_current_block().await.boxed()?;
        let metadata = at_block.metadata();
        let payload = RawPayload::from_bytes(&metadata, call_data).boxed()?;
        let response = api
            .tx()
            .await
            .boxed()?
            .sign_and_submit_then_watch_default(&payload, proxy_signer)
            .await
            .boxed()?;

        Ok(Response::transaction_submitted(response))
    }
}
