use crate::node_runtime;
use crate::node_runtime::runtime_types::{
    asset_hub_polkadot_runtime::RuntimeCall,
    frame_system::pallet::Call as SystemCall,
    pallet_staking_async::pallet::pallet::Call as StakingCall,
    pallet_staking_async::ValidatorPrefs,
    // pallet_staking_async_rc_client::pallet::Call as StakingRcClientCall,
    sp_arithmetic::per_things::Perbill,
};
use crate::utils::{map_payee, map_supported_proxy};
use subxt::{
    client::{ClientAtBlock, OnlineClientAtBlockImpl},
    utils::AccountId32,
    SubstrateConfig,
};
use suno_error::{Error, ResultExt};
use suno_primitives::{proxy::SupportedProxy, staking::Payee, tx::Bytes};

pub fn wrap_call_into_proxy(
    api: &ClientAtBlock<SubstrateConfig, OnlineClientAtBlockImpl<SubstrateConfig>>,
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

pub fn remark_with_event(value: Vec<u8>) -> RuntimeCall {
    RuntimeCall::System(SystemCall::remark_with_event { remark: value })
}

pub fn staking_chill() -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::chill {})
}

pub fn staking_bond(value: u128, payee: Payee) -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::bond {
        value,
        payee: map_payee(payee),
    })
}

pub fn staking_bond_extra(value: u128) -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::bond_extra {
        max_additional: value,
    })
}

pub fn staking_unbond(value: u128) -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::unbond { value })
}

pub fn staking_rebond(value: u128) -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::rebond { value })
}

pub fn staking_withdraw_unbonded() -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::withdraw_unbonded {
        num_slashing_spans: 0,
    })
}

pub fn staking_set_payee(payee: Payee) -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::set_payee {
        payee: map_payee(payee),
    })
}

pub fn staking_validate(commission: u32, blocked: bool) -> RuntimeCall {
    RuntimeCall::Staking(StakingCall::validate {
        prefs: ValidatorPrefs {
            commission: Perbill(commission),
            blocked,
        },
    })
}

// TODO: when ready
// pub fn staking_rc_client_set_keys(keys: Keys) -> RuntimeCall {
//     RuntimeCall::StakingRcClient(StakingRcClientCall::set_keys {
//         keys: keys.into_bytes(),
//         proof: vec![],
//         max_delivery_and_remote_execution_fee: None,
//     })
// }

// pub fn staking_rc_client_purge_keys() -> RuntimeCall {
//     RuntimeCall::StakingRcClient(StakingRcClientCall::purge_keys {
//         max_delivery_and_remote_execution_fee: None,
//     })
// }
