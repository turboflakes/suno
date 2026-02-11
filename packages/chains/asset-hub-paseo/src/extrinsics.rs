use crate::node_runtime;
use node_runtime::runtime_types::asset_hub_paseo_runtime::ProxyType;
use subxt::{utils::AccountId32, OnlineClient, SubstrateConfig};
use suno_error::Error;
use suno_primitives::tx::Bytes;

type Call = node_runtime::runtime_types::asset_hub_paseo_runtime::RuntimeCall;
type ProxyCall = node_runtime::runtime_types::pallet_proxy::pallet::Call;
type SystemCall = node_runtime::runtime_types::frame_system::pallet::Call;
type StakingCall = node_runtime::runtime_types::pallet_staking_async::pallet::pallet::Call;

pub fn wrap_call_into_proxy(
    api: &OnlineClient<SubstrateConfig>,
    call: Call,
    proxied_account: &AccountId32,
) -> Result<Bytes, Error> {
    let proxy_call = node_runtime::tx().proxy().proxy(
        proxied_account.clone().into(),
        Some(ProxyType::NonTransfer),
        call,
    );

    let payload = api.tx().call_data(&proxy_call)?;

    Ok(payload)
}

pub fn proxy(call: Call, proxied_account: &AccountId32) -> Call {
    Call::Proxy(ProxyCall::proxy {
        real: proxied_account.clone().into(),
        force_proxy_type: Some(ProxyType::NonTransfer),
        call: Box::new(call),
    })
}

pub fn remark_with_event(value: Vec<u8>) -> Call {
    Call::System(SystemCall::remark_with_event { remark: value })
}

pub fn chill() -> Call {
    Call::Staking(StakingCall::chill {})
}
