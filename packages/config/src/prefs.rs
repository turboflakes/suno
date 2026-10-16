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
    /// The validators and the collators picked from the TUI, by chain, in the same shape as the
    /// `chains` of the config file, so they can be moved there to be listed by the config.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    chains: Vec<HashMap<SupportedRuntime, PickedChain>>,
}

/// The validators and the collators picked from the TUI of a chain, as addresses in the format
/// of the network.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct PickedChain {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    validators: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    collators: Vec<String>,
}

/// A validator or a collator picked from the TUI, as a favourite, since it is not listed in the
/// config file. The `chain` is the relay chain of a validator, and the chain of a collator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PickedNode {
    pub chain: SupportedRuntime,
    pub stash: AccountId32,
}

impl PickedNode {
    /// The address of the stash, in the format of the network of the chain.
    fn address(&self) -> String {
        let account = sp_core::crypto::AccountId32::new(self.stash.0);
        account.to_ss58check_with_version(Ss58AddressFormat::custom(self.chain.account_format()))
    }
}

impl Prefs {
    pub(crate) fn picked_validators(&self) -> Vec<PickedNode> {
        self.picked(|chain| &chain.validators)
    }

    pub(crate) fn picked_collators(&self) -> Vec<PickedNode> {
        self.picked(|chain| &chain.collators)
    }

    fn picked(&self, list: impl Fn(&PickedChain) -> &Vec<String>) -> Vec<PickedNode> {
        self.chains
            .iter()
            .flatten()
            .flat_map(|(chain, picked)| {
                list(picked)
                    .iter()
                    .filter_map(|address| match address.parse::<AccountId32>() {
                        Ok(stash) => Some(PickedNode {
                            chain: *chain,
                            stash,
                        }),
                        Err(e) => {
                            warn!("Ignoring the picked address {}: {}", address, e);
                            None
                        }
                    })
            })
            .collect()
    }

    pub(crate) fn set_picked_validators(&mut self, nodes: Vec<PickedNode>) {
        self.set_picked(nodes, |chain| &mut chain.validators);
    }

    pub(crate) fn set_picked_collators(&mut self, nodes: Vec<PickedNode>) {
        self.set_picked(nodes, |chain| &mut chain.collators);
    }

    /// Replaces the list of the picked `nodes`, validators or collators, and keeps the other.
    fn set_picked(
        &mut self,
        nodes: Vec<PickedNode>,
        list: impl Fn(&mut PickedChain) -> &mut Vec<String>,
    ) {
        let mut chains: BTreeMap<SupportedRuntime, PickedChain> = BTreeMap::new();
        for (chain, picked) in self.chains.drain(..).flatten() {
            let entry = chains.entry(chain).or_default();
            entry.validators.extend(picked.validators);
            entry.collators.extend(picked.collators);
        }

        for picked in chains.values_mut() {
            list(picked).clear();
        }
        for node in nodes {
            let address = node.address();
            list(chains.entry(node.chain).or_default()).push(address);
        }

        self.chains = chains
            .into_iter()
            .filter(|(_, picked)| !picked.validators.is_empty() || !picked.collators.is_empty())
            .map(|(chain, picked)| HashMap::from([(chain, picked)]))
            .collect();
    }
}
