use crate::runtime::SupportedRuntime;
use serde::{Deserialize, Serialize};
use sp_core::crypto::{Ss58AddressFormat, Ss58Codec};
use std::collections::{BTreeMap, HashMap};
use subxt::utils::AccountId32;
use tracing::warn;

/// Runtime preferences saved from the TUI, layered on top of `config.yaml`
/// on load without ever touching the user's own file. Grows over time as
/// more things become editable from the TUI, each as its own optional
/// field so unrelated prefs are preserved on every write.
#[derive(Debug, Default, Serialize, Deserialize)]
pub(crate) struct Prefs {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) active_theme: Option<String>,
    /// The validators and the collators pinned from the TUI, by chain, in the same shape as the
    /// `chains` of the config file, so they can be moved there to be listed by the config.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    chains: Vec<HashMap<SupportedRuntime, PinnedChain>>,
    /// The features changed from the TUI, by name, overriding the ones of the config file.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(crate) features: BTreeMap<String, bool>,
}

/// The validators and the collators pinned from the TUI of a chain, as addresses in the format
/// of the network.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct PinnedChain {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    validators: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    collators: Vec<String>,
}

/// A validator or a collator pinned from the TUI, since it is not listed in the
/// config file. The `chain` is the relay chain of a validator, and the chain of a collator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedNode {
    pub chain: SupportedRuntime,
    pub stash: AccountId32,
}

impl PinnedNode {
    /// The address of the stash, in the format of the network of the chain.
    fn address(&self) -> String {
        let account = sp_core::crypto::AccountId32::new(self.stash.0);
        account.to_ss58check_with_version(Ss58AddressFormat::custom(self.chain.account_format()))
    }
}

impl Prefs {
    pub(crate) fn pinned_validators(&self) -> Vec<PinnedNode> {
        self.pinned(|chain| &chain.validators)
    }

    pub(crate) fn pinned_collators(&self) -> Vec<PinnedNode> {
        self.pinned(|chain| &chain.collators)
    }

    fn pinned(&self, list: impl Fn(&PinnedChain) -> &Vec<String>) -> Vec<PinnedNode> {
        self.chains
            .iter()
            .flatten()
            .flat_map(|(chain, pinned)| {
                list(pinned)
                    .iter()
                    .filter_map(|address| match address.parse::<AccountId32>() {
                        Ok(stash) => Some(PinnedNode {
                            chain: *chain,
                            stash,
                        }),
                        Err(e) => {
                            warn!("Ignoring the pinned address {}: {}", address, e);
                            None
                        }
                    })
            })
            .collect()
    }

    pub(crate) fn set_pinned_validators(&mut self, nodes: Vec<PinnedNode>) {
        self.set_pinned(nodes, |chain| &mut chain.validators);
    }

    pub(crate) fn set_pinned_collators(&mut self, nodes: Vec<PinnedNode>) {
        self.set_pinned(nodes, |chain| &mut chain.collators);
    }

    /// Replaces the list of the pinned `nodes`, validators or collators, and keeps the other.
    fn set_pinned(
        &mut self,
        nodes: Vec<PinnedNode>,
        list: impl Fn(&mut PinnedChain) -> &mut Vec<String>,
    ) {
        let mut chains: BTreeMap<SupportedRuntime, PinnedChain> = BTreeMap::new();
        for (chain, pinned) in self.chains.drain(..).flatten() {
            let entry = chains.entry(chain).or_default();
            entry.validators.extend(pinned.validators);
            entry.collators.extend(pinned.collators);
        }

        for pinned in chains.values_mut() {
            list(pinned).clear();
        }
        for node in nodes {
            let address = node.address();
            list(chains.entry(node.chain).or_default()).push(address);
        }

        self.chains = chains
            .into_iter()
            .filter(|(_, pinned)| !pinned.validators.is_empty() || !pinned.collators.is_empty())
            .map(|(chain, pinned)| HashMap::from([(chain, pinned)]))
            .collect();
    }
}
