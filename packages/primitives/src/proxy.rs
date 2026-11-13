use crate::call::Call;
use crate::display::{to_address_string, to_compact_string};
use crate::key::AccountBytes;
use crate::origin::Origin;

use std::collections::{BTreeSet, HashSet};
use subxt::utils::AccountId32;
use suno_config::{SupportedRuntime, CONFIG};

pub type Proxy = SupportedProxy;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// Represents the supported proxy types for a given runtime.
/// More context on the supported proxy types can be found in the Polkadot documentation:
/// https://docs.polkadot.com/node-infrastructure/run-a-validator/operational-tasks/staking-operator-proxy/#staking-operator-vs-staking-proxy
pub enum SupportedProxy {
    None,
    /// Staking proxy must be configured on Asset Hub and is limited to call extrinsics for
    /// staking operations on Asset Hub
    Staking,
    /// StakingOperator proxy must be configured on Asset Hub and is limited to a strict subset of
    /// the Staking proxy.
    StakingOperator,
    /// Collator proxy must be configured on the system chains where restricted colator operations
    /// are desired, such as `register_as_candidate`, `leave_intent`, `update_bond` or `take_candidate_slot`
    Collator,
    /// NonTransfer proxy must be configured on the system chains to execute `set_keys` or `purge_keys`
    /// extrinsics.
    NonTransfer,
}

impl SupportedProxy {
    pub fn as_short(&self) -> &'static str {
        match self {
            Self::None => "",
            Self::Staking => "S",
            Self::StakingOperator => "SO",
            Self::NonTransfer => "NT",
            Self::Collator => "C",
        }
    }

    pub fn can_call(&self, call: &Call) -> bool {
        match (self, call) {
            // NOTE: Staking proxy is limited to staking operations on Asset Hub
            (Self::Staking, Call::Bond { .. }) => true,
            (Self::Staking, Call::BondExtra { .. }) => true,
            (Self::Staking, Call::Unbond { .. }) => true,
            (Self::Staking, Call::Rebond { .. }) => true,
            (Self::Staking, Call::WithdrawUnbonded { .. }) => true,
            (Self::Staking, Call::SetPayee { .. }) => true,
            (Self::Staking, Call::Validate { .. }) => true,
            (Self::Staking, Call::Chill) => true,
            (Self::Staking, Call::SetKeys { .. }) => true,
            (Self::Staking, Call::PurgeKeys) => true,
            // TODO: implement Kick
            // (Self::Staking, Call::Kick) => true,
            // NOTE: StakingOperator proxy is limited to staking operations on Asset Hub
            (Self::StakingOperator, Call::Validate { .. }) => true,
            (Self::StakingOperator, Call::Chill) => true,
            (Self::StakingOperator, Call::SetKeys { .. }) => true,
            (Self::StakingOperator, Call::PurgeKeys) => true,
            // TODO: implement Kick
            // (Self::StakingOperator, Call::Kick) => true,
            // NOTE: NonTransfer proxy can make every staking operation on Asset Hub, and every
            // session_key and collator_selection operation on system parachains
            (Self::NonTransfer, Call::Bond { .. }) => true,
            (Self::NonTransfer, Call::BondExtra { .. }) => true,
            (Self::NonTransfer, Call::Unbond { .. }) => true,
            (Self::NonTransfer, Call::Rebond { .. }) => true,
            (Self::NonTransfer, Call::WithdrawUnbonded { .. }) => true,
            (Self::NonTransfer, Call::SetPayee { .. }) => true,
            (Self::NonTransfer, Call::Validate { .. }) => true,
            (Self::NonTransfer, Call::Chill) => true,
            (Self::NonTransfer, Call::SetKeys { .. }) => true,
            (Self::NonTransfer, Call::PurgeKeys) => true,
            (Self::NonTransfer, Call::SetSessionKeys { .. }) => true,
            (Self::NonTransfer, Call::PurgeSessionKeys) => true,
            (Self::NonTransfer, Call::RegisterAsCandidate) => true,
            (Self::NonTransfer, Call::LeaveIntent) => true,
            (Self::NonTransfer, Call::UpdateBond { .. }) => true,
            (Self::NonTransfer, Call::TakeCandidateSlot { .. }) => true,
            // NOTE: Collator proxy is limited to collator_selection operations on system parachains
            (Self::Collator, Call::RegisterAsCandidate) => true,
            (Self::Collator, Call::LeaveIntent) => true,
            (Self::Collator, Call::UpdateBond { .. }) => true,
            (Self::Collator, Call::TakeCandidateSlot { .. }) => true,
            _ => false,
        }
    }
}

impl std::fmt::Display for SupportedProxy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_short())
    }
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProxyKey {
    pub runtime: SupportedRuntime,
    pub proxy_type: SupportedProxy,
    pub proxy_account: Option<AccountBytes>,
}

impl ProxyKey {
    pub fn new(runtime: SupportedRuntime, proxy_type: SupportedProxy) -> Self {
        Self {
            runtime,
            proxy_type,
            proxy_account: None,
        }
    }

    pub fn with_proxy_account(mut self, proxy_account: Option<AccountBytes>) -> Self {
        self.proxy_account = proxy_account;
        self
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.runtime
    }

    pub fn proxy_type(&self) -> SupportedProxy {
        self.proxy_type
    }

    pub fn is_valid(&self) -> bool {
        self.proxy_type != SupportedProxy::None
    }

    pub fn is_staking_valid(&self) -> bool {
        (self.runtime == SupportedRuntime::AssetHubPolkadot
            || self.runtime == SupportedRuntime::AssetHubKusama
            || self.runtime == SupportedRuntime::AssetHubPaseo
            || self.runtime == SupportedRuntime::AssetHubWestend)
            && self.proxy_type == SupportedProxy::Staking
    }

    pub fn is_staking_operator_valid(&self) -> bool {
        (self.runtime == SupportedRuntime::AssetHubPolkadot
            || self.runtime == SupportedRuntime::AssetHubKusama
            || self.runtime == SupportedRuntime::AssetHubPaseo
            || self.runtime == SupportedRuntime::AssetHubWestend)
            && self.proxy_type == SupportedProxy::StakingOperator
    }

    pub fn is_non_transfer_valid(&self) -> bool {
        self.runtime.is_para_chain() && self.proxy_type == SupportedProxy::NonTransfer
    }

    pub fn is_collator_valid(&self) -> bool {
        self.runtime.is_para_chain() && self.proxy_type == SupportedProxy::Collator
    }
}

impl std::fmt::Display for ProxyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.proxy_type())
    }
}

/// What holds a proxy with an account: the validators, the collators and the nominators, and the
/// proxy types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProxyUsage {
    pub validators: usize,
    pub validator_types: BTreeSet<SupportedProxy>,
    pub collators: usize,
    pub collator_types: BTreeSet<SupportedProxy>,
    pub nominators: usize,
    pub nominator_types: BTreeSet<SupportedProxy>,
}

impl ProxyUsage {
    /// The counts only, e.g. `3 2c 1n`, or `0` when nothing holds a proxy.
    pub fn short(&self) -> String {
        let mut used = Vec::new();
        if self.validators > 0 {
            used.push(format!("{}v", self.validators));
        }
        if self.collators > 0 {
            used.push(format!("{}c", self.collators));
        }
        if self.nominators > 0 {
            used.push(format!("{}n", self.nominators));
        }
        if used.is_empty() {
            "0".to_string()
        } else {
            used.join(" ")
        }
    }
}

impl std::fmt::Display for ProxyUsage {
    /// E.g. `3 validators (S), 2 collators (C, NT), 1 nominators (S)`, or `0` when nothing holds a
    /// proxy.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let describe = |count: usize, name: &str, types: &BTreeSet<SupportedProxy>| {
            let types: Vec<_> = types.iter().map(|t| t.as_short()).collect();
            format!("{count} {name} ({})", types.join(", "))
        };
        let mut used = Vec::new();
        if self.validators > 0 {
            used.push(describe(
                self.validators,
                "validators",
                &self.validator_types,
            ));
        }
        if self.collators > 0 {
            used.push(describe(self.collators, "collators", &self.collator_types));
        }
        if self.nominators > 0 {
            used.push(describe(
                self.nominators,
                "nominators",
                &self.nominator_types,
            ));
        }
        if used.is_empty() {
            write!(f, "0")
        } else {
            write!(f, "{}", used.join(", "))
        }
    }
}

/// A proxy account a chain signs with, as listed in the TUI.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProxyState {
    pub runtime: SupportedRuntime,
    pub account: AccountId32,
    pub origin: Origin,
    pub usage: ProxyUsage,
}

impl ProxyState {
    /// The proxy accounts the chain `runtime` signs with.
    pub fn list(runtime: SupportedRuntime) -> Vec<Self> {
        let pinned = CONFIG.pinned_proxy_accounts(runtime);
        runtime
            .signer_account_ids()
            .into_iter()
            .map(|account| Self {
                runtime,
                origin: if pinned.contains(&account) {
                    Origin::Pinned
                } else {
                    Origin::Config
                },
                account,
                usage: ProxyUsage::default(),
            })
            .collect()
    }

    /// The proxy accounts of every chain that has them: the system chains configured, ordered
    /// by chain.
    pub fn list_all() -> Vec<Self> {
        let mut runtimes: Vec<SupportedRuntime> = CONFIG
            .chains()
            .iter()
            .flat_map(|chain| chain.keys().copied())
            .filter(|runtime| runtime.is_para_chain())
            .collect();
        runtimes.sort();
        runtimes.dedup();
        runtimes.into_iter().flat_map(Self::list).collect()
    }

    /// The address of the account, in the format of the network of the chain.
    pub fn address(&self) -> String {
        to_address_string(&self.account, self.runtime.account_format())
    }

    /// The address of the account, shortened to `size` characters at each end.
    pub fn compact_address(&self, size: usize) -> String {
        to_compact_string(&self.account, self.runtime.account_format(), size)
    }

    /// Whether it can be removed from the TUI, which is only so for the ones pinned from it.
    pub fn is_removable(&self) -> bool {
        self.origin.is_pinned()
    }

    /// The proxy types the `proxies` of a node hold with the account.
    fn held_by(&self, proxies: &HashSet<ProxyKey>) -> BTreeSet<SupportedProxy> {
        let account: AccountBytes = *self.account.as_ref();
        proxies
            .iter()
            // A stash without a proxy with the account is also kept, without a proxy type
            .filter(|p| {
                p.is_valid() && p.runtime == self.runtime && p.proxy_account == Some(account)
            })
            .map(|p| p.proxy_type)
            .collect()
    }

    /// Counts a validator with these `proxies`, when it holds a proxy with the account.
    pub fn track_validator(&mut self, proxies: &HashSet<ProxyKey>) {
        let types = self.held_by(proxies);
        if !types.is_empty() {
            self.usage.validators += 1;
            self.usage.validator_types.extend(types);
        }
    }

    /// Counts a nominator with these `proxies`, when it holds a proxy with the account.
    pub fn track_nominator(&mut self, proxies: &HashSet<ProxyKey>) {
        let types = self.held_by(proxies);
        if !types.is_empty() {
            self.usage.nominators += 1;
            self.usage.nominator_types.extend(types);
        }
    }

    /// Counts a collator with these `proxies`, when it holds a proxy with the account.
    pub fn track_collator(&mut self, proxies: &HashSet<ProxyKey>) {
        let types = self.held_by(proxies);
        if !types.is_empty() {
            self.usage.collators += 1;
            self.usage.collator_types.extend(types);
        }
    }
}

/// The valid proxies as a list, e.g. `[Staking,NonTransfer]`, or nothing when there are none.
pub fn proxies_to_string(proxies: &HashSet<ProxyKey>) -> String {
    let mut proxies = proxies
        .iter()
        .filter(|p| p.is_valid())
        .cloned()
        .collect::<Vec<_>>();

    if proxies.is_empty() {
        return String::new();
    }

    proxies.sort();
    format!(
        "[{}]",
        proxies
            .iter()
            .map(|p| p.to_string())
            .collect::<Vec<_>>()
            .join(",")
    )
}

/// The signer to make the `call` of a stash with the `proxies`, and its proxy type: the first
/// signer of the chain `runtime` that is a proxy of the stash able to make the call. When none is,
/// the first signer, without a proxy type. It is none when the chain has no signers.
pub fn select_proxy(
    proxies: &HashSet<ProxyKey>,
    runtime: SupportedRuntime,
    call: &Call,
) -> Option<(AccountId32, SupportedProxy)> {
    let signers = runtime.signer_account_ids();

    for signer in &signers {
        let proxy_account: AccountBytes = *signer.as_ref();
        let proxy = proxies.iter().find(|p| {
            p.runtime == runtime
                && p.proxy_account == Some(proxy_account)
                && p.proxy_type.can_call(call)
        });
        if let Some(proxy) = proxy {
            return Some((*signer, proxy.proxy_type));
        }
    }

    signers
        .first()
        .map(|signer| (*signer, SupportedProxy::None))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(runtime: SupportedRuntime, account: [u8; 32]) -> ProxyState {
        ProxyState {
            runtime,
            account: AccountId32::from(account),
            origin: Origin::Config,
            usage: ProxyUsage::default(),
        }
    }

    fn key(runtime: SupportedRuntime, proxy_type: SupportedProxy, account: [u8; 32]) -> ProxyKey {
        ProxyKey::new(runtime, proxy_type).with_proxy_account(Some(account))
    }

    #[test]
    fn a_stash_without_a_proxy_is_not_counted() {
        let runtime = SupportedRuntime::AssetHubPolkadot;
        let mut proxy = state(runtime, [1; 32]);
        let none = HashSet::from([key(runtime, SupportedProxy::None, [1; 32])]);
        proxy.track_validator(&none);
        proxy.track_collator(&none);
        proxy.track_nominator(&none);
        assert_eq!(proxy.usage, ProxyUsage::default());
        assert_eq!(proxy.usage.short(), "0");
    }

    #[test]
    fn only_the_proxies_of_the_account_and_the_chain_are_counted() {
        let runtime = SupportedRuntime::AssetHubPolkadot;
        let mut proxy = state(runtime, [1; 32]);
        let staking = HashSet::from([
            key(runtime, SupportedProxy::Staking, [1; 32]),
            key(runtime, SupportedProxy::StakingOperator, [1; 32]),
        ]);
        let other_account = HashSet::from([key(runtime, SupportedProxy::Staking, [2; 32])]);
        let other_chain = HashSet::from([key(
            SupportedRuntime::AssetHubKusama,
            SupportedProxy::Staking,
            [1; 32],
        )]);

        proxy.track_validator(&staking);
        proxy.track_validator(&staking);
        proxy.track_validator(&other_account);
        proxy.track_validator(&other_chain);
        proxy.track_collator(&HashSet::from([key(
            runtime,
            SupportedProxy::Collator,
            [1; 32],
        )]));

        assert_eq!(proxy.usage.validators, 2);
        proxy.track_nominator(&staking);
        assert_eq!(proxy.usage.collators, 1);
        assert_eq!(proxy.usage.nominators, 1);
        assert_eq!(proxy.usage.short(), "2v 1c 1n");
        assert_eq!(
            proxy.usage.to_string(),
            "2 validators (S, SO), 1 collators (C), 1 nominators (S, SO)"
        );
    }
}
