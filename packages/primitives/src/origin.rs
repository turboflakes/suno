/// Where a node, e.g. a collator or a validator, was listed from.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum Origin {
    /// Listed in the config file, it can carry hosts, commands and proxies.
    #[default]
    Config,
    /// Listed in the remote source the config points to, e.g. a validators URL.
    Remote,
    /// Pinned from the UI and persisted in the prefs file.
    Pinned,
    /// Nominated by one of the nominators the config or prefs lists.
    Nominated,
    /// Only found on-chain, listed when all of them are shown.
    OnChain,
}

impl Origin {
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Config => "▲",
            Self::Remote => "⚑",
            Self::Pinned => "●",
            Self::Nominated => "◆",
            Self::OnChain => "○",
        }
    }

    /// The next origin if toggled.
    pub fn toggle(self) -> Option<Self> {
        match self {
            Self::OnChain => Some(Self::Pinned),
            Self::Pinned => Some(Self::OnChain),
            Self::Config | Self::Remote | Self::Nominated => None,
        }
    }

    /// A label for the toggle action
    pub fn toggle_label(&self) -> Option<&'static str> {
        match self {
            Self::OnChain => Some("pin"),
            Self::Pinned => Some("unpin"),
            Self::Config | Self::Remote | Self::Nominated => None,
        }
    }

    /// Nodes the user follows, as opposed to the ones only found on-chain.
    pub fn is_tracked(&self) -> bool {
        !matches!(self, Self::OnChain)
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

    pub fn is_nominated(&self) -> bool {
        matches!(self, Self::Nominated)
    }
}

/// Every place a node is listed from, since the same node can be listed from more than one,
/// e.g. a validator in the config that is also nominated by a nominator.
///
/// A node found on-chain only is tracked by no other origin, so `Origin::OnChain` is only
/// held while no other origin is.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Origins(u8);

impl Origins {
    /// The origins by priority, the first one is the one displayed.
    const PRIORITY: [Origin; 5] = [
        Origin::Config,
        Origin::Remote,
        Origin::Pinned,
        Origin::Nominated,
        Origin::OnChain,
    ];

    fn bit(origin: Origin) -> u8 {
        1 << (origin as u8)
    }

    pub fn new(origin: Origin) -> Self {
        let mut origins = Self::default();
        origins.add(origin);
        origins
    }

    pub fn contains(&self, origin: Origin) -> bool {
        self.0 & Self::bit(origin) != 0
    }

    /// Adds the `origin`. Any other origin means the node is not only found on-chain.
    pub fn add(&mut self, origin: Origin) {
        if origin.is_tracked() {
            self.0 &= !Self::bit(Origin::OnChain);
        }
        self.0 |= Self::bit(origin);
    }

    /// Removes the `origin`. A node left with no origin is only found on-chain.
    pub fn remove(&mut self, origin: Origin) {
        self.0 &= !Self::bit(origin);
        if !self.is_tracked() {
            self.0 |= Self::bit(Origin::OnChain);
        }
    }

    /// The origins held, by priority.
    pub fn iter(&self) -> impl Iterator<Item = Origin> + '_ {
        Self::PRIORITY.into_iter().filter(|o| self.contains(*o))
    }

    /// The origin with the highest priority.
    pub fn primary(&self) -> Option<Origin> {
        self.iter().next()
    }

    /// Whether the node is followed, as opposed to only found on-chain.
    pub fn is_tracked(&self) -> bool {
        self.iter().any(|o| o.is_tracked())
    }

    /// Listed by the config, in the file or in its remote source.
    pub fn is_config(&self) -> bool {
        self.iter().any(|o| o.is_config())
    }

    pub fn is_pinned(&self) -> bool {
        self.contains(Origin::Pinned)
    }

    pub fn is_nominated(&self) -> bool {
        self.contains(Origin::Nominated)
    }

    /// Only found on-chain, no other origin follows it.
    pub fn is_onchain(&self) -> bool {
        !self.is_tracked()
    }

    /// The symbol of the origin with the highest priority.
    pub fn symbol(&self) -> &'static str {
        self.primary().map_or("", |o| o.symbol())
    }

    /// The symbols of every origin held, by priority.
    pub fn symbols(&self) -> String {
        self.iter().map(|o| o.symbol()).collect()
    }

    /// The origins if pinned, or if unpinned when it was pinned. The nodes listed by the config
    /// can't be toggled.
    pub fn toggle(&self) -> Option<Self> {
        if self.is_config() {
            return None;
        }
        let mut origins = *self;
        if self.is_pinned() {
            origins.remove(Origin::Pinned);
        } else {
            origins.add(Origin::Pinned);
        }
        Some(origins)
    }

    /// A label for the toggle action
    pub fn toggle_label(&self) -> Option<&'static str> {
        if self.is_config() {
            None
        } else if self.is_pinned() {
            Some("unpin")
        } else {
            Some("pin")
        }
    }
}

impl From<Origin> for Origins {
    fn from(origin: Origin) -> Self {
        Self::new(origin)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_single_origin_toggles_as_before() {
        let onchain = Origins::new(Origin::OnChain);
        let pinned = onchain.toggle().unwrap();
        assert_eq!(pinned, Origins::new(Origin::Pinned));
        assert_eq!(pinned.toggle(), Some(onchain));
        assert_eq!(Origins::new(Origin::Config).toggle(), None);
        assert_eq!(Origins::new(Origin::Remote).toggle(), None);
        assert_eq!(onchain.toggle_label(), Some("pin"));
        assert_eq!(pinned.toggle_label(), Some("unpin"));
        assert_eq!(Origins::new(Origin::Config).toggle_label(), None);
    }

    #[test]
    fn a_node_can_have_more_than_one_origin() {
        let mut origins = Origins::new(Origin::Config);
        origins.add(Origin::Nominated);
        assert!(origins.contains(Origin::Config));
        assert!(origins.contains(Origin::Nominated));
        assert!(origins.is_tracked());
        // The highest priority is displayed first
        assert_eq!(origins.primary(), Some(Origin::Config));
        assert_eq!(origins.symbols(), "▲◆");
        assert_eq!(origins.symbol(), "▲");
    }

    #[test]
    fn on_chain_is_only_held_while_nothing_else_follows_the_node() {
        let mut origins = Origins::new(Origin::OnChain);
        assert!(origins.is_onchain());

        origins.add(Origin::Nominated);
        assert!(!origins.contains(Origin::OnChain));
        assert!(origins.is_tracked());

        origins.remove(Origin::Nominated);
        assert!(origins.contains(Origin::OnChain));
        assert!(origins.is_onchain());
    }

    #[test]
    fn unpinning_a_nominated_node_keeps_it_tracked() {
        let mut origins = Origins::new(Origin::Nominated);
        assert_eq!(origins.toggle_label(), Some("pin"));

        origins.add(Origin::Pinned);
        assert_eq!(origins.toggle_label(), Some("unpin"));

        let unpinned = origins.toggle().unwrap();
        assert_eq!(unpinned, Origins::new(Origin::Nominated));
        assert!(unpinned.is_tracked());
    }

    #[test]
    fn a_node_of_the_config_can_not_be_toggled_even_if_nominated() {
        let mut origins = Origins::new(Origin::Remote);
        origins.add(Origin::Nominated);
        assert_eq!(origins.toggle(), None);
        assert_eq!(origins.toggle_label(), None);
    }

    #[test]
    fn only_the_origins_outside_the_config_can_be_toggled() {
        assert_eq!(Origin::OnChain.toggle(), Some(Origin::Pinned));
        assert_eq!(Origin::Pinned.toggle(), Some(Origin::OnChain));
        assert_eq!(Origin::Config.toggle(), None);
        assert_eq!(Origin::Remote.toggle(), None);
        assert_eq!(Origin::Nominated.toggle(), None);
    }

    #[test]
    fn the_legend_action_exists_when_the_origin_can_be_toggled() {
        for origin in [
            Origin::Config,
            Origin::Remote,
            Origin::Pinned,
            Origin::Nominated,
            Origin::OnChain,
        ] {
            assert_eq!(origin.toggle().is_some(), origin.toggle_label().is_some());
        }
    }
}
