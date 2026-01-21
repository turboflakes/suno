use crate::node_runtime;

type Call = node_runtime::runtime_types::asset_hub_polkadot_runtime::RuntimeCall;
type SystemCall = node_runtime::runtime_types::frame_system::pallet::Call;
type StakingCall = node_runtime::runtime_types::pallet_staking_async::pallet::pallet::Call;

pub fn remark_with_event(value: Vec<u8>) -> Call {
    Call::System(SystemCall::remark_with_event { remark: value })
}

pub fn chill() -> Call {
    Call::Staking(StakingCall::chill {})
}
