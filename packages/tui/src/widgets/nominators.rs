use crate::widgets::list_focus::ListFocus;
use crate::widgets::validators_detailed_group::{GROUP_HEADER_HEIGHT, PADDING};
use ratatui::widgets::TableState;
use std::collections::{BTreeMap, HashMap, HashSet};
use subxt::utils::AccountId32;
use suno_config::{PinnedNode, SupportedRuntime, CONFIG};
use suno_primitives::{
    balance::Balance,
    identity::Identity,
    key::AccountBytes,
    origin::{Origin, Origins},
    proxy::ProxyKey,
    staking::{Chunk, Nominations, Payee, StakeLedger},
    AccountKey, Nominator,
};
use suno_theme::Theme;

/// The nominators listed in the config, one row each.
#[derive(Debug, Default)]
pub struct NominatorsList {
    nominators: HashMap<AccountKey, Nominator>,
    nominators_order: Vec<AccountKey>,
    /// Nominators whose identity was requested already
    identity_requested: HashSet<AccountKey>,
    pub table_state: TableState,
    focus: ListFocus,
    pub scroll_offset: u16,
    viewport_height: u16,
    theme: Theme,
}

impl NominatorsList {
    /// Adds the nominator, or its origins to the one already listed for the same stash.
    fn add(&mut self, nominator: Nominator) {
        let key = nominator.key().clone();
        // A nominator listed from more than one place is listed once, with every origin
        if let Some(listed) = self.nominators.get_mut(&key) {
            for origin in nominator.origins.iter() {
                listed.origins.add(origin);
            }
            return;
        }
        // Keep the order sorted by relay chain (insertion order within a chain), so the
        // nominators of a chain are listed together and the selection order matches the order
        // of the groups displayed.
        let position = self
            .nominators_order
            .partition_point(|k| k.runtime.relay_chain() <= key.runtime.relay_chain());
        self.nominators_order.insert(position, key.clone());
        self.nominators.insert(key, nominator);
    }

    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = theme;
    }

    pub fn is_active(&self) -> bool {
        self.focus.is_active()
    }

    pub fn focus(&self) -> ListFocus {
        self.focus
    }

    pub fn set_focus(&mut self, focus: ListFocus) {
        self.focus = focus;
    }

    pub fn set_viewport_height(&mut self, height: u16) {
        self.viewport_height = height;
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

        // The nominators pinned from the TUI, of the chains configured, unless already listed
        for pinned in CONFIG.pinned_nominators() {
            if pinned.chain.is_asset_hub()
                && CONFIG
                    .chains()
                    .iter()
                    .any(|c| c.contains_key(&pinned.chain))
            {
                self.add(Nominator::with_origin(
                    pinned.chain,
                    pinned.stash,
                    Origin::Pinned,
                ));
            }
        }
        self.table_state
            .select((!self.nominators.is_empty()).then_some(0));
    }

    /// Pins the nominator, or unpins it when it was pinned. The nominators listed by the config
    /// can't be toggled. Returns the new origins of the nominator, if they changed.
    pub fn toggle_pinned(&mut self, key: &AccountKey) -> Option<Origins> {
        let nominator = self.nominators.get_mut(key)?;
        let origins = nominator.origins.toggle()?;
        nominator.origins = origins;
        Some(origins)
    }

    /// Adds a nominator pinned from the TUI. Returns its key, or none when it is already listed.
    pub fn add_pinned(
        &mut self,
        runtime: SupportedRuntime,
        stash: AccountId32,
    ) -> Option<AccountKey> {
        let key = AccountKey::new(runtime, stash);
        match self.nominators.get_mut(&key) {
            Some(listed) if listed.origins.is_pinned() => return None,
            Some(listed) => listed.origins.add(Origin::Pinned),
            None => {
                let selected = self.selected_key();
                self.add(Nominator::with_origin(runtime, stash, Origin::Pinned));
                self.restore_selection(selected);
            }
        }
        Some(key)
    }

    /// Unpins the nominator pinned from the TUI, which is no longer listed unless the config
    /// lists it too. Returns whether it was pinned.
    pub fn remove_pinned(&mut self, runtime: SupportedRuntime, stash: AccountId32) -> bool {
        let key = AccountKey::new(runtime, stash);
        let Some(nominator) = self
            .nominators
            .get_mut(&key)
            .filter(|n| n.origins.is_pinned())
        else {
            return false;
        };
        nominator.origins.remove(Origin::Pinned);
        if !nominator.is_tracked() {
            self.remove(&key);
        }
        true
    }

    /// The nominators pinned from the TUI, to be persisted.
    pub fn get_pinned(&self) -> Vec<PinnedNode> {
        self.nominators_iter()
            .filter(|n| n.origins.is_pinned())
            .map(|n| PinnedNode {
                chain: n.runtime(),
                stash: n.account.stash(),
            })
            .collect()
    }

    /// Removes the nominator from the list.
    pub fn remove(&mut self, key: &AccountKey) {
        let selected = self.selected_key().filter(|selected| selected != key);
        self.nominators.remove(key);
        self.identity_requested.remove(key);
        self.nominators_order.retain(|k| k != key);
        self.restore_selection(selected);
    }

    pub fn update_stake_ledger(&mut self, key: &AccountKey, ledger: StakeLedger) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.set_ledger(ledger);
        }
    }

    pub fn update_balance(&mut self, key: &AccountKey, balance: Balance) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.account.set_balance(balance);
        }
    }

    pub fn add_amount_to_stake_ledger(&mut self, key: &AccountKey, amount: u128) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.set_ledger(nominator.ledger.with_amount_added(amount));
        }
    }

    pub fn sub_chunk_from_stake_ledger(&mut self, key: &AccountKey, chunk: Chunk) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.set_ledger(nominator.ledger.with_chunk_unbonded(chunk));
        }
    }

    pub fn add_amount_to_balance(&mut self, key: &AccountKey, amount: u128) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.account.add_free_amount(amount);
        }
    }

    pub fn update_identity(&mut self, key: &AccountKey, identity: Identity) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.account.set_identity(Some(identity));
        }
    }

    /// The nominators whose identity wasn't requested yet, by relay chain. The identity is held
    /// on the People chain of the relay chain.
    pub fn get_keys_without_identity_requested(
        &self,
    ) -> HashMap<SupportedRuntime, Vec<AccountKey>> {
        let mut by_relay_chain: HashMap<SupportedRuntime, Vec<AccountKey>> = HashMap::new();
        for nominator in self.nominators_iter() {
            if nominator.identity().is_none() && !self.identity_requested.contains(nominator.key())
            {
                by_relay_chain
                    .entry(nominator.runtime().relay_chain())
                    .or_default()
                    .push(nominator.key().clone());
            }
        }
        by_relay_chain
    }

    pub fn mark_identity_requested(&mut self, keys: &[AccountKey]) {
        self.identity_requested.extend(keys.iter().cloned());
    }

    pub fn update_payee(&mut self, key: &AccountKey, payee: Payee) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.payee = payee;
        }
    }

    pub fn update_nominations(&mut self, key: &AccountKey, nominations: Option<Nominations>) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.set_nominations(nominations);
        }
    }

    pub fn add_proxy(&mut self, key: &AccountKey, proxy: ProxyKey) {
        if let Some(nominator) = self.nominators.get_mut(key) {
            nominator.proxies.insert(proxy);
        }
    }

    /// Forgets the proxies of the nominators of the chain `runtime`, e.g. when the proxy accounts
    /// changed, so they are fetched again.
    pub fn clear_proxies(&mut self, runtime: SupportedRuntime) {
        for nominator in self.nominators.values_mut() {
            if nominator.runtime() == runtime {
                nominator.proxies.clear();
            }
        }
    }

    fn selected_key(&self) -> Option<AccountKey> {
        self.table_state
            .selected()
            .and_then(|i| self.nominators_order.get(i))
            .cloned()
    }

    /// Selects `key` again after the order changed, or the first nominator when there is no `key`
    /// or it is gone.
    fn restore_selection(&mut self, key: Option<AccountKey>) {
        let index = key
            .and_then(|key| self.nominators_order.iter().position(|k| *k == key))
            .or_else(|| (!self.nominators_order.is_empty()).then_some(0));
        self.table_state.select(index);
        // Only follow the selection while it is being used, otherwise keep showing the top
        if self.focus.is_active() {
            self.ensure_selection_in_view();
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn is_listed(&self, nominator_key: &AccountKey) -> bool {
        self.nominators.contains_key(nominator_key)
    }

    pub fn nominators_iter(&self) -> impl Iterator<Item = &Nominator> {
        self.nominators_order
            .iter()
            .filter_map(|key| self.nominators.get(key))
    }

    pub fn is_empty(&self) -> bool {
        self.nominators.is_empty()
    }

    /// Returns true if any of the nominators listed has valid proxies to show.
    pub fn proxies_available(&self) -> bool {
        self.nominators_iter().any(|n| n.has_proxies())
    }

    /// The keys of the nominators of the chain `runtime`.
    pub fn keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        self.nominators_iter()
            .filter(|n| n.runtime() == runtime)
            .map(|n| n.key().clone())
            .collect()
    }

    /// The validators nominated by the nominators listed under the chain `runtime`, the relay
    /// chain. A validator nominated by more than one nominator is listed once.
    pub fn targets_by_relay_chain(&self, runtime: SupportedRuntime) -> HashSet<AccountBytes> {
        self.nominators_iter()
            .filter(|n| n.runtime().relay_chain() == runtime)
            .filter_map(|n| n.nominations.as_ref())
            .flat_map(|nominations| nominations.targets.iter().copied())
            .collect()
    }

    pub fn get_selected(&self) -> Option<&Nominator> {
        self.nominators
            .get(self.nominators_order.get(self.table_state.selected()?)?)
    }

    pub fn move_down(&mut self) {
        let len = self.nominators_order.len();
        if len == 0 {
            return;
        }
        let selected = self.table_state.selected().unwrap_or(0).min(len - 1);
        // Wrapped around to the beginning, start again from the top
        if selected + 1 >= len {
            self.scroll_offset = 0;
        }
        self.table_state.select(Some((selected + 1) % len));
        self.ensure_selection_in_view();
    }

    pub fn move_up(&mut self) {
        let len = self.nominators_order.len();
        if len == 0 {
            return;
        }
        let selected = self.table_state.selected().unwrap_or(0).min(len - 1);
        self.table_state.select(Some((selected + len - 1) % len));
        self.ensure_selection_in_view();
    }

    /// The nominators by relay chain, as displayed on the main window.
    pub fn get_nominators_grouped_by_runtime(&self) -> BTreeMap<SupportedRuntime, Vec<&Nominator>> {
        let mut grouped: BTreeMap<SupportedRuntime, Vec<&Nominator>> = BTreeMap::new();
        for nominator in self.nominators_iter() {
            grouped
                .entry(nominator.runtime().relay_chain())
                .or_default()
                .push(nominator);
        }
        grouped
    }

    /// Height needed by the main window to display every nominator without scrolling.
    pub fn total_detailed_group_height(&self) -> u16 {
        self.get_nominators_grouped_by_runtime()
            .values()
            .map(|n| GROUP_HEADER_HEIGHT + n.len() as u16 + PADDING)
            .sum()
    }

    /// Scrolls the main window to the selected nominator, if it is not in view.
    pub fn ensure_selection_in_view(&mut self) {
        // Nothing was rendered yet, so there is no viewport to scroll within
        if self.viewport_height == 0 {
            return;
        }
        let selected_y_position = self.selected_y_position();
        if selected_y_position < self.scroll_offset {
            self.scroll_offset = selected_y_position;
        } else if selected_y_position >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = selected_y_position - self.viewport_height + 1;
        }
    }

    /// The row where the selected nominator is displayed on the main window.
    fn selected_y_position(&self) -> u16 {
        let Some(selected) = self.get_selected() else {
            return 0;
        };
        let grouped = self.get_nominators_grouped_by_runtime();
        // Groups get a one row top padding when more than one is displayed
        let top_padding = u16::from(grouped.len() > 1);
        let mut group_y = 0u16;
        for nominators in grouped.values() {
            // Top padding + header + table header
            let first_row = group_y + top_padding + GROUP_HEADER_HEIGHT + 1;
            if let Some(index) = nominators.iter().position(|n| n.key() == selected.key()) {
                return first_row + index as u16;
            }
            group_y =
                group_y.saturating_add(GROUP_HEADER_HEIGHT + nominators.len() as u16 + PADDING);
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn the_events_update_the_ledger_and_the_balance_of_the_nominator_by_key() {
        let mut list = list();
        let key = nominator(2).key().clone();
        let other = nominator(3).key().clone();
        list.update_stake_ledger(&key, StakeLedger::new(100, 100, vec![]));

        list.add_amount_to_stake_ledger(&key, 50);
        list.sub_chunk_from_stake_ledger(&key, Chunk::new(7, 30));
        list.add_amount_to_balance(&key, 20);

        let updated = list.nominators_iter().find(|n| n.key() == &key).unwrap();
        assert_eq!(updated.ledger.active(), 120);
        assert_eq!(updated.ledger.unlocking(), &[Chunk::new(7, 30)]);
        assert_eq!(updated.free_balance(), 20);
        let untouched = list.nominators_iter().find(|n| n.key() == &other).unwrap();
        assert_eq!(untouched.ledger, StakeLedger::default());
        assert_eq!(untouched.free_balance(), 0);
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

    #[test]
    fn test_add_pinned_lists_once_and_is_persisted() {
        let mut list = list();
        let stash = AccountId32::from([9; 32]);

        let key = list.add_pinned(SupportedRuntime::AssetHubKusama, stash);
        assert!(key.is_some());
        assert!(list
            .add_pinned(SupportedRuntime::AssetHubKusama, stash)
            .is_none());

        // Only the pinned ones are persisted, not the ones of the config
        let pinned = list.get_pinned();
        assert_eq!(pinned.len(), 1);
        assert_eq!(pinned[0].stash, stash);
    }

    #[test]
    fn test_remove_pinned_only_removes_the_pinned_ones() {
        let mut list = list();
        let stash = AccountId32::from([9; 32]);
        let runtime = SupportedRuntime::AssetHubKusama;

        assert!(!list.remove_pinned(runtime, stash));
        list.add_pinned(runtime, stash);
        assert!(list.remove_pinned(runtime, stash));
        assert!(list.get_pinned().is_empty());
        assert!(!list.remove_pinned(runtime, stash));
    }

    #[test]
    fn the_selection_stays_on_its_nominator_when_one_is_added_before_it() {
        let mut list = list();
        list.table_state.select(Some(1));
        let selected = list.get_selected().unwrap().key().clone();

        // A nominator of a relay chain which is listed first
        list.add_pinned(
            SupportedRuntime::AssetHubPolkadot,
            AccountId32::from([9; 32]),
        );
        let first = list.nominators_iter().next().unwrap();
        assert_eq!(first.runtime(), SupportedRuntime::AssetHubPolkadot);
        assert_eq!(list.get_selected().unwrap().key(), &selected);
    }

    #[test]
    fn the_nominators_are_listed_by_relay_chain() {
        let mut list = list();
        list.add_pinned(
            SupportedRuntime::AssetHubPolkadot,
            AccountId32::from([9; 32]),
        );
        list.add_pinned(SupportedRuntime::AssetHubKusama, AccountId32::from([8; 32]));
        let chains: Vec<_> = list
            .nominators_iter()
            .map(|n| n.runtime().relay_chain())
            .collect();
        let mut sorted = chains.clone();
        sorted.sort();
        assert_eq!(chains, sorted);
    }

    #[test]
    fn the_first_nominator_is_selected_when_the_selected_one_is_removed() {
        let mut list = list();
        let stash = AccountId32::from([9; 32]);
        list.add_pinned(SupportedRuntime::AssetHubKusama, stash);
        let key = AccountKey::new(SupportedRuntime::AssetHubKusama, stash);
        let index = list.nominators_order.iter().position(|k| *k == key);
        list.table_state.select(index);

        assert!(list.remove_pinned(SupportedRuntime::AssetHubKusama, stash));
        assert_eq!(list.table_state.selected(), Some(0));
    }

    #[test]
    fn a_nominator_can_be_in_the_config_and_pinned() {
        let mut list = list();
        let stash = AccountId32::from([2; 32]);

        // Listed by the config, pinning adds the pin
        let key = list.add_pinned(SupportedRuntime::AssetHubKusama, stash);
        assert!(key.is_some());
        assert_eq!(list.nominators_iter().count(), 3);
        let listed = list
            .nominators_iter()
            .find(|n| n.account.stash() == stash)
            .unwrap();
        assert!(listed.origins.is_config() && listed.origins.is_pinned());
        assert_eq!(list.get_pinned().len(), 1);

        // Pinned twice is pinned once
        assert!(list
            .add_pinned(SupportedRuntime::AssetHubKusama, stash)
            .is_none());

        // Unpinning only drops the pin, the config still lists it
        assert!(list.remove_pinned(SupportedRuntime::AssetHubKusama, stash));
        assert_eq!(list.nominators_iter().count(), 3);
        assert!(list.get_pinned().is_empty());
    }

    #[test]
    fn the_origins_of_a_nominator_listed_twice_are_merged() {
        let mut list = list();
        list.add(Nominator::with_origin(
            SupportedRuntime::AssetHubKusama,
            AccountId32::from([1; 32]),
            Origin::Pinned,
        ));
        assert_eq!(list.nominators_iter().count(), 3);
        let listed = list.nominators_iter().next().unwrap();
        assert!(listed.origins.is_config() && listed.origins.is_pinned());
    }
}
