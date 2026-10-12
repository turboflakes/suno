use crate::node_runtime;
use crate::node_runtime::runtime_types::{
    coretime_polkadot_runtime::{RuntimeCall, SessionKeys},
    pallet_collator_selection::pallet::Call as CollatorSelectionCall,
    pallet_session::pallet::Call as SessionCall,
    sp_consensus_aura::sr25519::app_sr25519::Public as AuraPublic,
};
use crate::utils::map_supported_proxy;
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::AccountId32,
};
use suno_config::CustomConfig;
use suno_error::{Error, ResultExt};
use suno_primitives::{
    proxy::SupportedProxy,
    session::{AuraKey, Proof},
    tx::Bytes,
};

pub fn wrap_call_into_proxy(
    api: &ClientAtBlock<CustomConfig, OnlineClientAtBlockImpl<CustomConfig>>,
    call: RuntimeCall,
    proxied_account: &AccountId32,
    supported_proxy: SupportedProxy,
) -> Result<Bytes, Error> {
    let proxy_type = map_supported_proxy(supported_proxy);
    let proxy_call = node_runtime::tx()
        .proxy()
        .proxy((*proxied_account).into(), proxy_type, call);

    let payload = api.tx().call_data(&proxy_call).boxed()?;

    Ok(payload)
}

pub fn session_set_keys(aura_key: AuraKey, proof: Proof) -> RuntimeCall {
    RuntimeCall::Session(SessionCall::set_keys {
        keys: SessionKeys {
            aura: AuraPublic(aura_key.into_bytes()),
        },
        proof: proof.into_bytes(),
    })
}

pub fn session_purge_keys() -> RuntimeCall {
    RuntimeCall::Session(SessionCall::purge_keys)
}

pub fn collator_selection_register_as_candidate() -> RuntimeCall {
    RuntimeCall::CollatorSelection(CollatorSelectionCall::register_as_candidate {})
}

pub fn collator_selection_leave_intent() -> RuntimeCall {
    RuntimeCall::CollatorSelection(CollatorSelectionCall::leave_intent {})
}

pub fn collator_selection_update_bond(new_deposit: u128) -> RuntimeCall {
    RuntimeCall::CollatorSelection(CollatorSelectionCall::update_bond { new_deposit })
}

pub fn collator_selection_take_candidate_slot(deposit: u128, target: AccountId32) -> RuntimeCall {
    RuntimeCall::CollatorSelection(CollatorSelectionCall::take_candidate_slot { deposit, target })
}
