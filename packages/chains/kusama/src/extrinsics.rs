use crate::node_runtime;
use node_runtime::runtime_types::{
    kusama_runtime_constants::proxy::ProxyType,
    pallet_proxy::pallet::Call as ProxyCall,
    pallet_session::pallet::Call as SessionCall,
    polkadot_primitives::v8::{
        assignment_app::Public as AssignmentPublic, validator_app::Public as ValidatorPublic,
    },
    sp_authority_discovery::app::Public as AuthorityDiscoveryPublic,
    sp_consensus_babe::app::Public as BabePublic,
    sp_consensus_beefy::ecdsa_crypto::Public as BeefyPublic,
    sp_consensus_grandpa::app::Public as GrandpaPublic,
    staging_kusama_runtime::{RuntimeCall, SessionKeys},
};
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::AccountId32,
    SubstrateConfig,
};
use suno_error::{Error, ResultExt};
use suno_primitives::session::Keys;

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

pub fn session_set_keys(keys: Keys) -> RuntimeCall {
    RuntimeCall::Session(SessionCall::set_keys {
        keys: SessionKeys {
            grandpa: GrandpaPublic(keys.grandpa_bytes),
            babe: BabePublic(keys.babe_bytes),
            para_validator: ValidatorPublic(keys.para_validator_bytes),
            para_assignment: AssignmentPublic(keys.para_assignment_bytes),
            authority_discovery: AuthorityDiscoveryPublic(keys.authority_discovery_bytes),
            beefy: BeefyPublic(keys.beefy_bytes),
        },
        proof: vec![],
    })
}
