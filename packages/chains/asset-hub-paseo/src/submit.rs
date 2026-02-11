use super::node_runtime;
use node_runtime::runtime_types::asset_hub_paseo_runtime::ProxyType;
use subxt::{utils::AccountId32, OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::Keypair;
use suno_error::Error;
use suno_primitives::{tx::payload_from_bytes, Response};

type Call = node_runtime::runtime_types::asset_hub_paseo_runtime::RuntimeCall;

pub async fn sign_and_submit_call_data(
    api: &OnlineClient<SubstrateConfig>,
    proxy_signer: &Keypair,
    call_data: Vec<u8>,
) -> Result<Response, Error> {
    let payload = payload_from_bytes(call_data);

    let response = api
        .tx()
        .sign_and_submit_then_watch_default(&payload, proxy_signer)
        .await?;

    Ok(Response::transaction_progress(response))
}

pub async fn _submit_as_proxy(
    api: &OnlineClient<SubstrateConfig>,
    call: Call,
    proxied_account: &AccountId32,
    proxy_signer: &Keypair,
) -> Result<Response, Error> {
    let proxy_call = node_runtime::tx().proxy().proxy(
        proxied_account.clone().into(),
        Some(ProxyType::NonTransfer),
        call,
    );

    let response = api
        .tx()
        .sign_and_submit_then_watch_default(&proxy_call, proxy_signer)
        .await?;

    Ok(Response::transaction_progress(response))
}
