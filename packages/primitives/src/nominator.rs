use crate::{
    account::{Account, AccountDisplay},
    identity::Identity,
    key::AccountKey,
    origin::{Origin, Origins},
    proxy::{proxies_to_string, ProxyKey},
    staking::{Nominations, Payee, StakeLedger},
};
use std::collections::HashSet;
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

/// An account that nominates validators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nominator {
    /// Stash account that nominates validators.
    pub account: Account,
    /// Every place the nominator is listed from, e.g. the config and also pinned from the UI.
    pub origins: Origins,
    /// The nominations made by the nominator.
    pub nominations: Option<Nominations>,
    /// Stake ledger of the nominator.
    pub ledger: StakeLedger,
    /// Payee of the nominator.
    pub payee: Payee,
    /// Proxies of the nominator.
    pub proxies: HashSet<ProxyKey>,
}

impl Nominator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: Account::new(runtime, stash),
            origins: Origins::new(Origin::Config),
            nominations: None,
            ledger: StakeLedger::default(),
            payee: Payee::None,
            proxies: HashSet::new(),
        }
    }

    pub fn with_origin(runtime: SupportedRuntime, stash: AccountId32, origin: Origin) -> Self {
        Self {
            origins: Origins::new(origin),
            ..Self::new(runtime, stash)
        }
    }

    pub fn key(&self) -> &AccountKey {
        self.account.account_key()
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.account.runtime()
    }

    pub fn identity(&self) -> &Option<Identity> {
        self.account.identity()
    }

    pub fn is_tracked(&self) -> bool {
        self.origins.is_tracked()
    }

    pub fn is_nominating(&self) -> bool {
        self.nominations
            .as_ref()
            .is_some_and(|nominations| !nominations.targets.is_empty())
    }

    /// Whether any of the proxies is valid, so there is something to show.
    pub fn has_proxies(&self) -> bool {
        self.proxies.iter().any(|p| p.is_valid())
    }

    pub fn proxies_as_str(&self) -> String {
        proxies_to_string(&self.proxies)
    }

    /// The stake being unbonded, which can't be withdrawn yet at `era`.
    pub fn unlocking(&self, era: u32) -> u128 {
        self.ledger
            .unlocking()
            .iter()
            .filter(|c| c.era > era)
            .map(|c| c.value)
            .sum()
    }

    /// The stake unbonded, which can be withdrawn at `era`.
    pub fn unlocked(&self, era: u32) -> u128 {
        self.ledger
            .unlocking()
            .iter()
            .filter(|c| c.era <= era)
            .map(|c| c.value)
            .sum()
    }

    pub fn targets_count(&self) -> usize {
        self.nominations
            .as_ref()
            .map_or(0, |nominations| nominations.targets.len())
    }
}

impl AccountDisplay for Nominator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }

    fn account_format(&self) -> u16 {
        self.account.account_format()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::staking::Chunk;

    fn nominator() -> Nominator {
        Nominator::new(SupportedRuntime::AssetHubKusama, AccountId32::from([1; 32]))
    }

    #[test]
    fn a_new_nominator_is_listed_from_the_config_and_nominates_nothing() {
        let nominator = nominator();
        assert_eq!(nominator.origins, Origins::new(Origin::Config));
        assert!(nominator.is_tracked());
        assert!(!nominator.is_nominating());
        assert_eq!(nominator.targets_count(), 0);
    }

    #[test]
    fn a_nominator_with_targets_is_nominating() {
        let mut nominator = nominator();
        nominator.nominations = Some(Nominations {
            targets: vec![[2; 32], [3; 32]],
            submitted_in: 10,
            suppressed: false,
        });
        assert!(nominator.is_nominating());
        assert_eq!(nominator.targets_count(), 2);
    }

    #[test]
    fn the_stake_is_unlocking_until_its_era_and_unlocked_after() {
        let mut nominator = nominator();
        nominator.ledger = StakeLedger::new(100, 160, vec![Chunk::new(10, 20), Chunk::new(12, 40)]);
        assert_eq!(nominator.unlocking(9), 60);
        assert_eq!(nominator.unlocked(9), 0);
        assert_eq!(nominator.unlocking(10), 40);
        assert_eq!(nominator.unlocked(10), 20);
        assert_eq!(nominator.unlocking(12), 0);
        assert_eq!(nominator.unlocked(12), 60);
    }

    #[test]
    fn the_origin_can_be_chosen() {
        let nominator = Nominator::with_origin(
            SupportedRuntime::AssetHubKusama,
            AccountId32::from([1; 32]),
            Origin::Pinned,
        );
        assert_eq!(nominator.origins, Origins::new(Origin::Pinned));
    }
}
