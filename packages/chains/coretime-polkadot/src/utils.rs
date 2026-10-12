use crate::node_runtime::runtime_types::coretime_polkadot_runtime::ProxyType;
use suno_primitives::proxy::SupportedProxy;

/// Helper function to map SupportedProxy to ProxyType
pub fn map_supported_proxy(proxy: SupportedProxy) -> Option<ProxyType> {
    match proxy {
        SupportedProxy::None | SupportedProxy::Staking | SupportedProxy::StakingOperator => None,
        SupportedProxy::NonTransfer => Some(ProxyType::NonTransfer),
        SupportedProxy::Collator => Some(ProxyType::Collator),
    }
}
