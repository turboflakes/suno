/// Where a node, e.g. a collator or a validator, was listed from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origin {
    /// Listed in the config file, it can carry hosts, commands and proxies.
    #[default]
    Config,
    /// Picked from the UI and persisted in the prefs file, only monitored.
    Picked,
    /// Only found on-chain, listed when all of them are shown.
    OnChain,
}

impl Origin {
    /// Nodes the user follows, as opposed to the ones only found on-chain.
    pub fn is_tracked(&self) -> bool {
        *self != Self::OnChain
    }

    /// Nodes listed in the config file, which can't be untracked from the UI.
    pub fn is_config(&self) -> bool {
        *self == Self::Config
    }
}
