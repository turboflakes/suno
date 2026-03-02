use crate::node_runtime::{self, runtime_types::pallet_staking_async::ValidatorPrefs};
use node_runtime::runtime_types::{
    asset_hub_polkadot_runtime::ProxyType, pallet_staking_async::RewardDestination,
    sp_arithmetic::per_things::Perbill,
};
use subxt::{utils::AccountId32, OnlineClient, SubstrateConfig};
use suno_error::Error;
use suno_primitives::{staking::Payee, tx::Bytes};

type RuntimeCall = node_runtime::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;
type ProxyCall = node_runtime::runtime_types::pallet_proxy::pallet::Call;
type SystemCall = node_runtime::runtime_types::frame_system::pallet::Call;
type StakingCall = node_runtime::runtime_types::pallet_staking_async::pallet::pallet::Call;

pub fn wrap_call_into_proxy(
    api: &OnlineClient<SubstrateConfig>,
    call: RuntimeCall,
    proxied_account: &AccountId32,
) -> Result<Bytes, Error> {
    let proxy_call = node_runtime::tx().proxy().proxy(
        proxied_account.clone().into(),
        Some(ProxyType::Staking),
        call,
    );

    let payload = api.tx().call_data(&proxy_call)?;

    Ok(payload)
}

pub fn proxy(call: RuntimeCall, proxied_account: &AccountId32) -> RuntimeCall {
    RuntimeCall::Proxy(ProxyCall::proxy {
        real: proxied_account.clone().into(),
        force_proxy_type: Some(ProxyType::Staking),
        call: Box::new(call),
    })
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

// Helper function to map Payee to RewardDestination
fn map_payee(payee: Payee) -> RewardDestination<AccountId32> {
    match payee {
        Payee::None => RewardDestination::None,
        Payee::Account(account) => RewardDestination::Account(account),
        Payee::Stash => RewardDestination::Stash,
        Payee::Staked => RewardDestination::Staked,
    }
}
