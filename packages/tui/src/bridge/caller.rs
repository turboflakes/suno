use async_trait::async_trait;
use subxt::ext::codec::Decode;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::{AccountId32, MultiSignature},
    OnlineClient,
};
use subxt_signer::sr25519::Keypair;
use suno_config::{
    transactions::{build_signed_extrinsic, build_signing_payload, signed_extrinsic_bytes},
    CustomConfig, Runtime,
};
use suno_error::{Error, ResultExt};
use suno_primitives::{call::Call, proxy::SupportedProxy, tx::Bytes, Response};

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

        // subxt's typed `create_v4_signable`/`create_v5_signable` (used by
        // `sign_and_submit_then_watch_default`) are driven by `CustomConfig`'s
        // compile-time-fixed `TransactionExtensions` tuple, which doesn't cover several
        // extensions this chain actually declares (e.g. `AuthorizeCall`, `CheckWeight`,
        // `CheckNonZeroSender`) — that fails outright with a `NotFound` error. Build and
        // sign the extrinsic manually instead, driven by the chain's real metadata, exactly
        // like the Vault QR flow does — see `suno_qrcode::build::encode_extensions`.
        let account_id = AccountId32(proxy_signer.public_key().0);

        let (signing_payload, extra) =
            build_signing_payload(&at_block, &account_id, call_data)
                .await
                .map_err(|e| Error::Other(e.to_string()))?;

        let signature = proxy_signer.sign(&signing_payload);

        let extrinsic_bytes =
            signed_extrinsic_bytes(&account_id.0, &signature.0, &extra, call_data);

        let response = at_block
            .tx()
            .from_bytes(extrinsic_bytes)
            .submit_and_watch()
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

        // This signature always comes from an air-gapped signer (Polkadot Vault), which was
        // shown a transaction built by `suno_qrcode::build::build_transaction_qrcode`. That
        // function encodes extensions by walking the chain's actual metadata (not subxt's
        // compile-time-fixed `TransactionExtensions` tuple), so we must reconstruct the final
        // extrinsic the same way here — see `build_signed_extrinsic` — otherwise the bytes we
        // submit won't match what was actually signed, and the chain rejects it as a bad
        // signature.
        let sig_bytes = match extract_signature(signature)? {
            MultiSignature::Sr25519(sig) => sig,
            _ => {
                return Err(Error::Other(
                    "Only Sr25519 signatures are supported".to_string(),
                ))
            }
        };

        let extrinsic_bytes =
            build_signed_extrinsic(&at_block, proxy_signer, call_data, &sig_bytes)
                .await
                .map_err(|e| Error::Other(e.to_string()))?;

        let response = at_block
            .tx()
            .from_bytes(extrinsic_bytes)
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
