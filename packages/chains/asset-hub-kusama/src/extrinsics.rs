use super::node_runtime;
pub use node_runtime::staking::storage::types::nominators::Nominators;

type Call = node_runtime::runtime_types::asset_hub_kusama_runtime::RuntimeCall;
type StakingCall = node_runtime::runtime_types::pallet_staking_async::pallet::pallet::Call;

pub fn chill() -> Call {
    Call::Staking(StakingCall::chill {})
}
