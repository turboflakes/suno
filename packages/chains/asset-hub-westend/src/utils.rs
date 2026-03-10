use crate::node_runtime::runtime_types::pallet_staking_async::RewardDestination;
use subxt::utils::AccountId32;
use suno_primitives::staking::Payee;

// Helper function to map RewardDestination to Payee
pub fn map_reward_destination(dest: RewardDestination<AccountId32>) -> Payee {
    match dest {
        RewardDestination::None | RewardDestination::Controller => Payee::None,
        RewardDestination::Account(account) => Payee::Account(account),
        RewardDestination::Stash => Payee::Stash,
        RewardDestination::Staked => Payee::Staked,
    }
}
