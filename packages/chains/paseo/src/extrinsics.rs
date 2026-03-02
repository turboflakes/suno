use crate::node_runtime;
use node_runtime::runtime_types::{
    paseo_runtime::SessionKeys,
    paseo_runtime_constants::proxy::ProxyType,
    polkadot_primitives::v8::{
        assignment_app::Public as AssignmentPublic, validator_app::Public as ValidatorPublic,
    },
    sp_authority_discovery::app::Public as AuthorityDiscoveryPublic,
    sp_consensus_babe::app::Public as BabePublic,
    sp_consensus_beefy::ecdsa_crypto::Public as BeefyPublic,
    sp_consensus_grandpa::app::Public as GrandpaPublic,
};
use subxt::{utils::AccountId32, OnlineClient, SubstrateConfig};
use suno_error::{Error, ResultExt};
use suno_primitives::{session::Keys, tx::Bytes};

type RuntimeCall = node_runtime::runtime_types::paseo_runtime::RuntimeCall;
type ProxyCall = node_runtime::runtime_types::pallet_proxy::pallet::Call;
type SessionCall = node_runtime::runtime_types::pallet_session::pallet::Call;

pub fn wrap_call_into_proxy(
    api: &OnlineClient<SubstrateConfig>,
    call: RuntimeCall,
    proxied_account: &AccountId32,
) -> Result<Bytes, Error> {
    let proxy_call = node_runtime::tx().proxy().proxy(
        proxied_account.clone().into(),
        Some(ProxyType::NonTransfer),
        call,
    );

    let payload = api.tx().call_data(&proxy_call).boxed()?;

    Ok(payload)
}

pub fn proxy(call: RuntimeCall, proxied_account: &AccountId32) -> RuntimeCall {
    RuntimeCall::Proxy(ProxyCall::proxy {
        real: proxied_account.clone().into(),
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
