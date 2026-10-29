/// Where a node, e.g. a collator or a validator, was listed from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origin {
    /// Listed in the config file, it can carry hosts, commands and proxies.
    #[default]
    Config,
    /// Listed in the remote source the config points to, e.g. a validators URL.
    Remote,
    /// Pinned from the UI and persisted in the prefs file, only monitored.
    Pinned,
    /// Only found on-chain, listed when all of them are shown.
    OnChain,
}

impl Origin {
    /// Nodes the user follows, as opposed to the ones only found on-chain.
    pub fn is_tracked(&self) -> bool {
        *self != Self::OnChain
    }

    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Config => "▲",
            Self::Remote => "⚑",
            Self::Pinned => "●",
            Self::OnChain => "○",
        }
    }

    /// The next origin if toggled.
    pub fn toggle(self) -> Option<Self> {
        match self {
            Self::OnChain => Some(Self::Pinned),
            Self::Pinned => Some(Self::OnChain),
            Self::Config | Self::Remote => None,
        }
    }

    /// A label for the toggle action
    pub fn toggle_label(&self) -> Option<&'static str> {
        match self {
            Self::OnChain => Some("pin"),
            Self::Pinned => Some("unpin"),
            Self::Config | Self::Remote => None,
        }
    }

    /// Nodes listed by the config, in the file or in its remote source, which can't be
    /// untracked from the UI.
    pub fn is_config(&self) -> bool {
        matches!(self, Self::Config | Self::Remote)
    }

    pub fn is_onchain(&self) -> bool {
        matches!(self, Self::OnChain)
    }

    pub fn is_pinned(&self) -> bool {
        matches!(self, Self::Pinned)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_the_origins_outside_the_config_can_be_toggled() {
        assert_eq!(Origin::OnChain.toggle(), Some(Origin::Pinned));
        assert_eq!(Origin::Pinned.toggle(), Some(Origin::OnChain));
        assert_eq!(Origin::Config.toggle(), None);
        assert_eq!(Origin::Remote.toggle(), None);
    }

    #[test]
    fn the_legend_action_exists_when_the_origin_can_be_toggled() {
        for origin in [
            Origin::Config,
            Origin::Remote,
            Origin::Pinned,
            Origin::OnChain,
        ] {
            assert_eq!(origin.toggle().is_some(), origin.toggle_label().is_some());
        }
    }
}
