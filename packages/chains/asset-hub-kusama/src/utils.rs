use crate::node_runtime::runtime_types::{
    asset_hub_kusama_runtime::ProxyType, pallet_staking_async::RewardDestination,
};
use subxt::utils::AccountId32;
use suno_primitives::{proxy::SupportedProxy, staking::Payee};

/// Helper function to map RewardDestination to Payee
pub fn map_reward_destination(dest: RewardDestination<AccountId32>) -> Payee {
    match dest {
        RewardDestination::None | RewardDestination::Controller => Payee::None,
        RewardDestination::Account(account) => Payee::Account(account),
        RewardDestination::Stash => Payee::Stash,
        RewardDestination::Staked => Payee::Staked,
    }
}

/// Helper function to map Payee to RewardDestination
pub fn map_payee(payee: Payee) -> RewardDestination<AccountId32> {
    match payee {
        Payee::None => RewardDestination::None,
        Payee::Account(account) => RewardDestination::Account(account),
        Payee::Stash => RewardDestination::Stash,
        Payee::Staked => RewardDestination::Staked,
    }
}

/// Helper function to map SupportedProxy to ProxyType
pub fn map_supported_proxy(proxy: SupportedProxy) -> Option<ProxyType> {
    match proxy {
        SupportedProxy::None => None,
        SupportedProxy::NonTransfer => Some(ProxyType::NonTransfer),
        SupportedProxy::Staking => Some(ProxyType::Staking),
        SupportedProxy::StakingOperator => Some(ProxyType::StakingOperator),
    }
}
