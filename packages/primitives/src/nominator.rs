use crate::{
    account::{Account, AccountDisplay},
    identity::Identity,
    key::AccountKey,
    origin::Origin,
    proxy::ProxyKey,
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
    /// The origin of the nominator, e.g. whether it's listed in the config or pinned from the UI.
    pub origin: Origin,
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
            origin: Origin::Config,
            nominations: None,
            ledger: StakeLedger::default(),
            payee: Payee::None,
            proxies: HashSet::new(),
        }
    }

    pub fn with_origin(runtime: SupportedRuntime, stash: AccountId32, origin: Origin) -> Self {
        Self {
            origin,
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
        self.origin.is_tracked()
    }

    pub fn is_nominating(&self) -> bool {
        self.nominations
            .as_ref()
            .is_some_and(|nominations| !nominations.targets.is_empty())
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

    fn nominator() -> Nominator {
        Nominator::new(SupportedRuntime::AssetHubKusama, AccountId32::from([1; 32]))
    }

    #[test]
    fn a_new_nominator_is_listed_from_the_config_and_nominates_nothing() {
        let nominator = nominator();
        assert_eq!(nominator.origin, Origin::Config);
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
    fn the_origin_can_be_chosen() {
        let nominator = Nominator::with_origin(
            SupportedRuntime::AssetHubKusama,
            AccountId32::from([1; 32]),
            Origin::Pinned,
        );
        assert_eq!(nominator.origin, Origin::Pinned);
    }
}
