use crate::call::Call;
use suno_config::SupportedRuntime;

pub type Proxy = SupportedProxy;

#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
/// Represents the supported proxy types for a given runtime.
/// More context on the supported proxy types can be found in the Polkadot documentation:
/// https://docs.polkadot.com/node-infrastructure/run-a-validator/operational-tasks/staking-operator-proxy/#staking-operator-vs-staking-proxy
pub enum SupportedProxy {
    None,
    /// NonTransfer proxy must be configured on the Relay chain and is limited to SetKeys/PurgeKeys
    /// extrinsics on the Relay chain
    NonTransfer,
    /// Staking proxy must be configured on Asset Hub and is limited to call extrinsics for
    /// staking operations on Asset Hub
    Staking,
    /// StakingOperator proxy must be configured on Asset Hub and is limited to a strict subset of
    /// the Staking proxy.
    StakingOperator,
}

impl SupportedProxy {
    pub fn as_short(&self) -> &'static str {
        match self {
            Self::None => "×",
            Self::NonTransfer => "NT",
            Self::Staking => "S",
            Self::StakingOperator => "SO",
        }
    }

    pub fn can_call(&self, call: &Call) -> bool {
        match (self, call) {
            // NOTE: NonTransfer proxy is limited to SetKeys/PurgeKeys on the Relay chain
            (Self::NonTransfer, Call::SetKeys { .. }) => true,
            // (Self::NonTransfer, Call::PurgeKeys) => true,
            // NOTE: Staking proxy is limited to staking operations on Asset Hub
            (Self::Staking, Call::Bond { .. }) => true,
            (Self::Staking, Call::BondExtra { .. }) => true,
            (Self::Staking, Call::Unbond { .. }) => true,
            (Self::Staking, Call::Rebond { .. }) => true,
            (Self::Staking, Call::WithdrawUnbonded { .. }) => true,
            (Self::Staking, Call::SetPayee { .. }) => true,
            (Self::Staking, Call::Validate { .. }) => true,
            (Self::Staking, Call::Chill) => true,
            // TODO: implement Kick and PurgeKeys
            // (Self::Staking, Call::Kick) => true,
            // NOTE: StakingOperator proxy is limited to staking operations on Asset Hub
            (Self::StakingOperator, Call::Validate { .. }) => true,
            (Self::StakingOperator, Call::Chill) => true,
            (Self::StakingOperator, Call::SetKeys { .. }) => true,
            // TODO: implement Kick and PurgeKeys
            // (Self::StakingOperator, Call::Kick) => true,
            // (Self::StakingOperator, Call::PurgeKeys) => true,
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
    pub proxy: SupportedProxy,
}

impl ProxyKey {
    pub fn new(runtime: SupportedRuntime, proxy: SupportedProxy) -> Self {
        Self { runtime, proxy }
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.runtime
    }

    pub fn proxy(&self) -> SupportedProxy {
        self.proxy
    }

    pub fn is_staking_valid(&self) -> bool {
        (self.runtime == SupportedRuntime::AssetHubPolkadot
            || self.runtime == SupportedRuntime::AssetHubKusama
            || self.runtime == SupportedRuntime::AssetHubPaseo
            || self.runtime == SupportedRuntime::AssetHubWestend)
            && self.proxy == SupportedProxy::Staking
    }

    pub fn is_staking_operator_valid(&self) -> bool {
        (self.runtime == SupportedRuntime::AssetHubPolkadot
            || self.runtime == SupportedRuntime::AssetHubKusama
            || self.runtime == SupportedRuntime::AssetHubPaseo
            || self.runtime == SupportedRuntime::AssetHubWestend)
            && self.proxy == SupportedProxy::StakingOperator
    }

    pub fn is_non_transfer_valid(&self) -> bool {
        (self.runtime == SupportedRuntime::Polkadot
            || self.runtime == SupportedRuntime::Kusama
            || self.runtime == SupportedRuntime::Paseo
            || self.runtime == SupportedRuntime::Westend)
            && self.proxy == SupportedProxy::NonTransfer
    }
}

impl std::fmt::Display for ProxyKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.proxy())
    }
}
