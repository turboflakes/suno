use crate::call::Call;
use crate::key::AccountBytes;
use std::collections::HashSet;
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

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
            // NOTE: NonTransfer proxy is limited to session_key operations on system parachains
            (Self::NonTransfer, Call::SetSessionKeys { .. }) => true,
            (Self::NonTransfer, Call::PurgeSessionKeys) => true,
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
