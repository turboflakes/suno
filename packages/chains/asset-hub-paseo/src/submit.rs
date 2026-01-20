use super::node_runtime;
use node_runtime::runtime_types::asset_hub_paseo_runtime::ProxyType;
use subxt::{utils::AccountId32, OnlineClient, SubstrateConfig};
use subxt_signer::sr25519::Keypair;
use suno_error::Error;
use suno_primitives::Response;

type Call = node_runtime::runtime_types::asset_hub_paseo_runtime::RuntimeCall;

pub async fn submit_as_proxy(
    api: &OnlineClient<SubstrateConfig>,
    call: Call,
    proxied_account: AccountId32,
    password: Option<String>,
) -> Result<Response, Error> {
    let proxy_signer: Keypair = suno_signer::load_keypair(password)?;

    let proxy_call = node_runtime::tx().proxy().proxy(
        proxied_account.into(),
        Some(ProxyType::NonTransfer),
        call,
    );

    let response = api
        .tx()
        .sign_and_submit_then_watch_default(&proxy_call, &proxy_signer)
        .await?;

    Ok(Response::transaction_progress(response))
}
