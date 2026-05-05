use crate::node_runtime;
use crate::utils::map_session_keys_from_keys;
use node_runtime::runtime_types::{
    kusama_runtime_constants::proxy::ProxyType, pallet_proxy::pallet::Call as ProxyCall,
    pallet_session::pallet::Call as SessionCall, staging_kusama_runtime::RuntimeCall,
};
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::AccountId32,
    SubstrateConfig,
};
use suno_error::{Error, ResultExt};
use suno_primitives::session::{Keys, Proof};

type Bytes = Vec<u8>;

pub fn wrap_call_into_proxy(
    api: &ClientAtBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
    call: RuntimeCall,
    proxied_account: &AccountId32,
) -> Result<Bytes, Error> {
    let proxy_call = node_runtime::tx().proxy().proxy(
        (*proxied_account).into(),
        Some(ProxyType::NonTransfer),
        call,
    );

    let payload = api.tx().call_data(&proxy_call).boxed()?;

    Ok(payload)
}

pub fn proxy(call: RuntimeCall, proxied_account: &AccountId32) -> RuntimeCall {
    RuntimeCall::Proxy(ProxyCall::proxy {
        real: (*proxied_account).into(),
        force_proxy_type: Some(ProxyType::Staking),
        call: Box::new(call),
    })
}

pub fn session_set_keys(keys: Keys, proof: Proof) -> RuntimeCall {
    let session_keys = map_session_keys_from_keys(&keys);
    RuntimeCall::Session(SessionCall::set_keys {
        keys: session_keys,
        proof: proof.into_bytes(),
    })
}

pub fn session_purge_keys() -> RuntimeCall {
    RuntimeCall::Session(SessionCall::purge_keys {})
}
