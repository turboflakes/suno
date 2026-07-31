use async_trait::async_trait;
use subxt::ext::codec::Decode;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::{AccountId32, MultiSignature},
    OnlineClient,
};
use subxt_signer::sr25519::Keypair;
use suno_config::{CustomConfig, CustomExtrinsicParamsBuilder, Runtime};
use suno_error::{Error, ResultExt};
use suno_primitives::{
    call::Call,
    proxy::SupportedProxy,
    tx::{Bytes, RawPayload},
    Response,
};

#[async_trait]
pub trait RuntimeCaller {
    fn build_call_data(
        &self,
        api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
        stash: &AccountId32,
        call: Call,
        supported_proxy: SupportedProxy,
    ) -> Result<Bytes, Error>;

    async fn sign_and_submit_call_data(
        &self,
        api: &OnlineClient<CustomConfig>,
        proxy_signer: &Keypair,
        call_data: &[u8],
    ) -> Result<Response, Error>;

    async fn submit_call_data_with_signature(
        &self,
        api: &OnlineClient<CustomConfig>,
        proxy_signer: &AccountId32,
        call_data: &[u8],
        signature: &[u8],
    ) -> Result<Response, Error>;
}

#[async_trait]
impl RuntimeCaller for Runtime {
    fn build_call_data(
        &self,
        api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
        stash: &AccountId32,
        call: Call,
        supported_proxy: SupportedProxy,
    ) -> Result<Bytes, Error> {
        match &self {
            Runtime::AssetHubPolkadot => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_unbond(amount);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_rebond(amount);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::WithdrawUnbonded { .. } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Chill => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_chill();
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetKeys { keys, proof } => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_rc_client_set_keys(
                        keys, proof,
                    );
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::PurgeKeys => {
                    let rc = suno_asset_hub_polkadot::extrinsics::staking_rc_client_purge_keys();
                    suno_asset_hub_polkadot::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::AssetHubKusama => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_unbond(amount);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_rebond(amount);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::WithdrawUnbonded { .. } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Chill => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_chill();
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetKeys { keys, proof } => {
                    let rc =
                        suno_asset_hub_kusama::extrinsics::staking_rc_client_set_keys(keys, proof);
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::PurgeKeys => {
                    let rc = suno_asset_hub_kusama::extrinsics::staking_rc_client_purge_keys();
                    suno_asset_hub_kusama::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::AssetHubPaseo => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_unbond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_rebond(amount);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::WithdrawUnbonded { .. } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Chill => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_chill();
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetKeys { keys, proof } => {
                    let rc =
                        suno_asset_hub_paseo::extrinsics::staking_rc_client_set_keys(keys, proof);
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::PurgeKeys => {
                    let rc = suno_asset_hub_paseo::extrinsics::staking_rc_client_purge_keys();
                    suno_asset_hub_paseo::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            Runtime::AssetHubWestend => match call {
                Call::Bond { amount, payee, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_bond(amount, payee);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::BondExtra { amount, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_bond_extra(amount);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Unbond { amount, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_unbond(amount);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Rebond { amount, .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_rebond(amount);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::WithdrawUnbonded { .. } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_withdraw_unbonded();
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetPayee { payee } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_set_payee(payee);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Validate {
                    commission,
                    blocked,
                } => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_validate(
                        commission.deconstruct(),
                        blocked,
                    );
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::Chill => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_chill();
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::SetKeys { keys, proof } => {
                    let rc =
                        suno_asset_hub_westend::extrinsics::staking_rc_client_set_keys(keys, proof);
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                Call::PurgeKeys => {
                    let rc = suno_asset_hub_westend::extrinsics::staking_rc_client_purge_keys();
                    suno_asset_hub_westend::wrap_call_into_proxy(api, rc, stash, supported_proxy)
                }
                _ => Err(Error::UnsupportedCall(call.to_string())),
            },
            _ => Err(Error::UnsupportedRuntime(*self)),
        }
    }

    async fn sign_and_submit_call_data(
        &self,
        api: &OnlineClient<CustomConfig>,
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

    async fn submit_call_data_with_signature(
        &self,
        api: &OnlineClient<CustomConfig>,
        proxy_signer: &AccountId32,
        call_data: &[u8],
        signature: &[u8],
    ) -> Result<Response, Error> {
        let at_block = api.at_current_block().await.boxed()?;
        let metadata = at_block.metadata();
        let payload = RawPayload::from_bytes(&metadata, call_data).boxed()?;

        let nonce = at_block.tx().account_nonce(proxy_signer).await.boxed()?;
        let params = CustomExtrinsicParamsBuilder::new().nonce(nonce).build();

        let mut signable = at_block
            .tx()
            .create_signable_offline(&payload, params)
            .boxed()?;

        let signature = extract_signature(signature)?;

        let response = signable
            .sign_with_account_and_signature(proxy_signer, &signature)
            .boxed()?
            .submit_and_watch()
            .await
            .boxed()?;

        Ok(Response::transaction_submitted(response))
    }
}

fn extract_signature(bytes: &[u8]) -> Result<MultiSignature, Error> {
    let mut input = bytes;

    MultiSignature::decode(&mut input).map_err(Error::InvalidSignature)
}
