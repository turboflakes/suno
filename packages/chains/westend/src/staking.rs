use super::node_runtime;

type Call = node_runtime::runtime_types::westend_runtime::RuntimeCall;
type StakingCall = node_runtime::runtime_types::pallet_staking::pallet::pallet::Call;

pub fn chill() -> Call {
    Call::Staking(StakingCall::chill {})
}
