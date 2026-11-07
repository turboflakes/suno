use ratatui::widgets::TableState;
use std::collections::HashSet;
use suno_config::{SupportedRuntime, CONFIG};
use suno_primitives::{key::AccountBytes, staking::Nominations, AccountKey, Nominator};
use suno_theme::Theme;

/// The nominators listed in the config, one row each.
#[derive(Debug, Default)]
pub struct NominatorsList {
    nominators: Vec<Nominator>,
    pub(crate) table_state: TableState,
    is_active: bool,
    theme: Theme,
}

impl NominatorsList {
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }

    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn on_init(&mut self) {
        if !CONFIG.features().nominators_enabled() {
            return;
        }

        for chain in CONFIG.chains().iter() {
            for (runtime, chain_config) in chain {
                if !runtime.is_asset_hub() {
                    continue;
                }
                for node in &chain_config.nominators {
                    self.add(Nominator::new(*runtime, node.stash()));
                }
            }
        }
        self.table_state
            .select((!self.nominators.is_empty()).then_some(0));
    }

    /// Adds the nominator, unless the same stash is already listed for the chain.
    fn add(&mut self, nominator: Nominator) {
        if self.nominators.iter().all(|n| n.key() != nominator.key()) {
            self.nominators.push(nominator);
        }
    }

    pub fn nominators_iter(&self) -> impl Iterator<Item = &Nominator> {
        self.nominators.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.nominators.is_empty()
    }

    /// The keys of the nominators of the chain `runtime`.
    pub fn keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        self.nominators
            .iter()
            .filter(|n| n.runtime() == runtime)
            .map(|n| n.key().clone())
            .collect()
    }

    /// The validators nominated by the nominators listed under the chain `runtime`, the relay
    /// chain. A validator nominated by more than one nominator is listed once.
    pub fn targets_by_relay_chain(&self, runtime: SupportedRuntime) -> HashSet<AccountBytes> {
        self.nominators
            .iter()
            .filter(|n| n.runtime().relay_chain() == runtime)
            .filter_map(|n| n.nominations.as_ref())
            .flat_map(|nominations| nominations.targets.iter().copied())
            .collect()
    }

    pub fn get_selected(&self) -> Option<&Nominator> {
        self.nominators.get(self.table_state.selected()?)
    }

    pub fn update_nominations(&mut self, key: &AccountKey, nominations: Option<Nominations>) {
        if let Some(nominator) = self.nominators.iter_mut().find(|n| n.key() == key) {
            nominator.nominations = nominations;
        }
    }

    pub fn move_down(&mut self) {
        let len = self.nominators.len();
        if len == 0 {
            return;
        }
        let selected = self.table_state.selected().unwrap_or(0).min(len - 1);
        self.table_state.select(Some((selected + 1) % len));
    }

    pub fn move_up(&mut self) {
        let len = self.nominators.len();
        if len == 0 {
            return;
        }
        let selected = self.table_state.selected().unwrap_or(0).min(len - 1);
        self.table_state.select(Some((selected + len - 1) % len));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use subxt::utils::AccountId32;

    fn nominator(byte: u8) -> Nominator {
        Nominator::new(
            SupportedRuntime::AssetHubKusama,
            AccountId32::from([byte; 32]),
        )
    }

    fn list() -> NominatorsList {
        let mut list = NominatorsList::default();
        list.add(nominator(1));
        list.add(nominator(2));
        list.add(nominator(3));
        list.table_state.select(Some(0));
        list
    }

    #[test]
    fn the_same_stash_is_listed_once() {
        let mut list = list();
        list.add(nominator(2));
        assert_eq!(list.nominators_iter().count(), 3);
    }

    #[test]
    fn the_selection_wraps_around() {
        let mut list = list();
        list.move_up();
        assert_eq!(list.table_state.selected(), Some(2));
        list.move_down();
        assert_eq!(list.table_state.selected(), Some(0));
    }

    #[test]
    fn the_nominations_are_updated_by_key() {
        let mut list = list();
        let key = nominator(2).key().clone();
        let nominations = Nominations {
            targets: vec![[9; 32]],
            submitted_in: 5,
            suppressed: false,
        };
        list.update_nominations(&key, Some(nominations.clone()));
        let updated = list.nominators_iter().find(|n| n.key() == &key).unwrap();
        assert_eq!(updated.nominations, Some(nominations));
        assert!(list
            .nominators_iter()
            .filter(|n| n.key() != &key)
            .all(|n| n.nominations.is_none()));
    }

    #[test]
    fn the_targets_are_nominated_once_by_relay_chain() {
        let mut list = list();
        let key = |byte: u8| nominator(byte).key().clone();
        let nominations = |targets: Vec<[u8; 32]>| Nominations {
            targets,
            submitted_in: 1,
            suppressed: false,
        };
        // Two nominators share a target
        list.update_nominations(&key(1), Some(nominations(vec![[10; 32], [11; 32]])));
        list.update_nominations(&key(2), Some(nominations(vec![[11; 32], [12; 32]])));
        // The third one nominates nothing yet
        list.update_nominations(&key(3), None);

        let targets = list.targets_by_relay_chain(SupportedRuntime::Kusama);
        assert_eq!(targets, HashSet::from([[10; 32], [11; 32], [12; 32]]));
        assert!(list
            .targets_by_relay_chain(SupportedRuntime::Polkadot)
            .is_empty());
    }

    #[test]
    fn the_keys_are_filtered_by_chain() {
        let list = list();
        assert_eq!(
            list.keys_by_runtime(SupportedRuntime::AssetHubKusama).len(),
            3
        );
        assert!(list
            .keys_by_runtime(SupportedRuntime::AssetHubPolkadot)
            .is_empty());
    }

    #[test]
    fn an_empty_list_does_not_move() {
        let mut list = NominatorsList::default();
        list.move_down();
        list.move_up();
        assert_eq!(list.table_state.selected(), None);
        assert!(list.get_selected().is_none());
    }
}
