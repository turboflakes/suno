use crate::widgets::collators_detailed_group::{GROUP_HEADER_HEIGHT, PADDING};
use crate::widgets::collators_detailed_list::group_header_height;
use crate::widgets::list_focus::{ListFocus, VISIBLE_ROWS_MARGIN};
use ratatui::widgets::TableState;
use sp_arithmetic::traits::Zero;
use std::{
    collections::{BTreeMap, HashMap, HashSet},
    time::{SystemTime, UNIX_EPOCH},
};
use subxt::utils::AccountId32;
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
use suno_primitives::{
    balance::Balance,
    collator::{Collator, CollatorStatus},
    identity::Identity,
    proxy::ProxyKey,
    AccountDisplay, AccountKey,
};

type CollatorKey = AccountKey;
type Amount = u128;
type AccountBytes = [u8; 32];

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CollatorsView {
    Group,
    #[default]
    List,
}

/// A group of collators in the detailed view displayed, and the row of its first collator.
struct RowGroup<'a> {
    first_row: u16,
    collators: Vec<&'a Collator>,
}

#[derive(Debug)]
pub struct CollatorsList {
    pub collators: HashMap<CollatorKey, Collator>,
    pub collators_order: Vec<CollatorKey>,
    pub table_state: TableState,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    focus: ListFocus,
    view: CollatorsView,
    masked: bool,
    /// Collators only found on-chain whose data was requested already, since it is only
    /// fetched for the ones displayed.
    requested: HashSet<CollatorKey>,
    /// Whether the window with this list is the one displayed.
    visible: bool,
    navigated: bool,
}

impl Default for CollatorsList {
    fn default() -> Self {
        Self {
            collators: HashMap::new(),
            collators_order: Vec::new(),
            table_state: TableState::default(),
            scroll_offset: 0,
            viewport_height: 0,
            focus: ListFocus::default(),
            view: CollatorsView::default(),
            masked: true,
            requested: HashSet::new(),
            visible: false,
            navigated: false,
        }
    }
}

impl CollatorsList {
    pub fn add_collator(&mut self, collator: Collator) {
        let key = collator.key().clone();
        if !self.collators.contains_key(&key) {
            // Keep the order sorted by runtime (config order within a runtime) so the
            // selection order matches the order of the groups displayed in both views.
            let position = self
                .collators_order
                .partition_point(|k| k.runtime <= key.runtime);
            self.collators_order.insert(position, key.clone());
        }
        self.collators.insert(key, collator);
    }

    /// Adds the on-chain `accounts` of `runtime` that aren't known yet as untracked collators,
    /// returning the keys of the ones just added.
    fn add_onchain_collators(
        &mut self,
        runtime: SupportedRuntime,
        accounts: impl IntoIterator<Item = AccountBytes>,
    ) -> Vec<CollatorKey> {
        let selected = self.selected_key();
        let mut added = Vec::new();
        for bytes in accounts {
            let key = CollatorKey::from_bytes(runtime, bytes);
            if self.collators.contains_key(&key) {
                continue;
            }
            self.add_collator(Collator::new_on_chain(runtime, AccountId32::from(bytes)));
            added.push(key);
        }
        // New collators are inserted in order, which shifts the selected index
        if !added.is_empty() {
            self.restore_selection(selected);
        }
        added
    }

    /// Removes the untracked collators of `runtime` that are no longer part of the on-chain
    /// `keep` set. Tracked collators are never removed.
    pub fn prune_untracked(&mut self, runtime: SupportedRuntime, keep: &HashSet<AccountBytes>) {
        let stale: Vec<CollatorKey> = self
            .collators
            .iter()
            .filter(|(key, c)| {
                !c.is_tracked() && key.runtime == runtime && !keep.contains(&key.bytes)
            })
            .map(|(key, _)| key.clone())
            .collect();
        if stale.is_empty() {
            return;
        }

        let selected = self.selected_key().filter(|key| !stale.contains(key));
        for key in &stale {
            self.collators.remove(key);
            self.requested.remove(key);
        }
        self.collators_order.retain(|key| !stale.contains(key));
        self.restore_selection(selected);
    }

    pub fn on_init(&mut self) {
        let chains = CONFIG.chains();
        for chain in chains.iter() {
            for (chain_name, chain_config) in chain {
                for collator in &chain_config.collators {
                    match collator {
                        NodeConfig::Address(stash) => {
                            self.add_collator(Collator::new(*chain_name, *stash));
                        }
                        NodeConfig::Detailed {
                            stash,
                            host_rpc,
                            ssh,
                            commands,
                        } => {
                            let mut collator = Collator::new(*chain_name, *stash);
                            collator.host_rpc = host_rpc.clone();
                            collator.ssh = ssh.clone();
                            if let Some(cmds) = commands {
                                collator.commands = cmds.clone();
                            }
                            self.add_collator(collator);
                        }
                    }
                }
            }
        }
        if !self.collators.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    /// Returns an iterator of collators in display order
    pub fn collators_iter(&self) -> impl Iterator<Item = &Collator> {
        self.collators_order
            .iter()
            .filter_map(move |key| self.collators.get(key))
    }

    // Returns true if any validator has proxies available
    pub fn proxies_available(&self) -> bool {
        self.collators_order
            .iter()
            .any(|key| self.collators.get(key).is_some_and(|c| c.has_proxies()))
    }

    // Helper method to get collator by table index
    pub fn get_collator_by_index(&self, index: usize) -> Option<&Collator> {
        self.collators_order
            .get(index)
            .and_then(|key| self.collators.get(key))
    }

    pub fn get_collator_by_index_cloned(&self, index: usize) -> Option<Collator> {
        self.get_collator_by_index(index).cloned()
    }

    pub fn get_collator_keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        self.collators_order
            .iter()
            .filter(|key| key.runtime == runtime)
            .cloned()
            .collect()
    }

    /// Keys of the collators with a row in the viewport of the detailed view displayed, plus a
    /// a few rows of margin above and below, so the data is there before the row is scrolled to.
    fn get_visible_keys(&self) -> Vec<CollatorKey> {
        // Not displayed, or nothing was rendered yet
        if !self.visible || self.viewport_height == 0 {
            return Vec::new();
        }

        let margin = VISIBLE_ROWS_MARGIN;
        let top = self.scroll_offset.saturating_sub(margin);
        let bottom = self
            .scroll_offset
            .saturating_add(self.viewport_height)
            .saturating_add(margin);

        let mut keys = Vec::new();
        for group in self.get_row_groups() {
            for (i, collator) in group.collators.iter().enumerate() {
                let row = group.first_row.saturating_add(i as u16);
                if row >= top && row < bottom {
                    keys.push(collator.key().clone());
                }
            }
        }
        keys
    }

    /// The collators only found on-chain that are visible and whose data wasn't requested yet,
    /// by runtime. They are marked as requested, so they are returned once.
    pub fn take_unrequested_visible(&mut self) -> HashMap<SupportedRuntime, Vec<CollatorKey>> {
        let mut by_runtime: HashMap<SupportedRuntime, Vec<CollatorKey>> = HashMap::new();
        for key in self.get_visible_keys() {
            let is_on_chain = self.collators.get(&key).is_some_and(|c| !c.is_tracked());
            if is_on_chain && self.requested.insert(key.clone()) {
                by_runtime.entry(key.runtime).or_default().push(key);
            }
        }
        by_runtime
    }

    /// Same as [`Self::get_collator_keys_by_runtime`], only for the tracked collators.
    pub fn get_tracked_collator_keys_by_runtime(
        &self,
        runtime: SupportedRuntime,
    ) -> Vec<CollatorKey> {
        self.get_collator_keys_by_runtime(runtime)
            .into_iter()
            .filter(|key| self.collators.get(key).is_some_and(|c| c.is_tracked()))
            .collect()
    }

    /// Keys to sync periodically: the tracked collators and the on-chain only ones visible.
    pub fn get_tracked_and_visible_keys_by_runtime(
        &self,
        runtime: SupportedRuntime,
    ) -> Vec<CollatorKey> {
        let visible = self.get_visible_keys();
        self.get_collator_keys_by_runtime(runtime)
            .into_iter()
            .filter(|key| {
                self.collators.get(key).is_some_and(|c| c.is_tracked()) || visible.contains(key)
            })
            .collect()
    }

    pub fn get_collators_grouped_by_runtime(&self) -> BTreeMap<SupportedRuntime, Vec<&Collator>> {
        let mut grouped: BTreeMap<SupportedRuntime, Vec<&Collator>> = BTreeMap::new();

        for key in &self.collators_order {
            if let Some(collator) = self.collators.get(key) {
                grouped
                    .entry(collator.runtime())
                    .or_default()
                    .push(collator);
            }
        }

        grouped
    }

    /// Same as [`Self::get_collators_grouped_by_runtime`], but grouped by relay chain
    /// instead of by parachain, so collators from every system chain of a relay (AssetHub,
    /// BridgeHub, Coretime, Collectives, People, ...) are listed together.
    pub fn get_collators_grouped_by_relay_chain(
        &self,
    ) -> BTreeMap<SupportedRuntime, Vec<&Collator>> {
        let mut grouped: BTreeMap<SupportedRuntime, Vec<&Collator>> = BTreeMap::new();

        for key in &self.collators_order {
            if let Some(collator) = self.collators.get(key) {
                grouped
                    .entry(collator.runtime().relay_chain())
                    .or_default()
                    .push(collator);
            }
        }

        grouped
    }

    pub fn total_detailed_group_height(&self) -> u16 {
        let grouped = self.get_collators_grouped_by_runtime();

        grouped
            .values()
            .map(|c| GROUP_HEADER_HEIGHT + c.len() as u16 + PADDING)
            .sum()
    }

    pub fn set_viewport_height(&mut self, height: u16) {
        self.viewport_height = height;
    }

    pub fn get_selected_ref(&self) -> Option<&Collator> {
        self.table_state
            .selected()
            .and_then(|i| self.get_collator_by_index(i))
    }

    // Scroll to the selected collator if it's not in view
    pub fn ensure_selection_in_view(&mut self) {
        // Nothing was rendered yet, so there is no viewport to scroll within
        if self.viewport_height == 0 {
            return;
        }

        let selected_y_position = self.get_selected_y_position();

        if selected_y_position < self.scroll_offset {
            self.scroll_offset = selected_y_position;
        } else if selected_y_position >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = selected_y_position - self.viewport_height + 1;
        }
    }

    // Determine the Y position of the current collator selection
    fn get_selected_y_position(&self) -> u16 {
        let selected_ref = self.get_selected_ref();

        self.get_row_groups()
            .iter()
            .find_map(|group| {
                let idx = group
                    .collators
                    .iter()
                    .position(|c| Some(*c) == selected_ref)?;
                Some(group.first_row + idx as u16)
            })
            .unwrap_or(0)
    }

    /// The groups of collators as laid out in the detailed view displayed: by parachain in the
    /// group view, and by relay chain, with a header per parachain, in the list view.
    fn get_row_groups(&self) -> Vec<RowGroup<'_>> {
        let groups: Vec<(u16, Vec<&Collator>)> = match self.view {
            CollatorsView::Group => self
                .get_collators_grouped_by_runtime()
                .into_values()
                .map(|collators| (GROUP_HEADER_HEIGHT, collators))
                .collect(),
            CollatorsView::List => self
                .get_collators_grouped_by_relay_chain()
                .into_values()
                .map(|collators| (group_header_height(&collators), collators))
                .collect(),
        };

        // Groups get a one row top padding when more than one is displayed
        let top_padding = if groups.len() > 1 { 1 } else { 0 };
        let mut group_y = 0u16;
        groups
            .into_iter()
            .map(|(header_height, collators)| {
                // Top padding + header + table header
                let first_row = group_y + top_padding + header_height + 1;
                group_y = group_y.saturating_add(header_height + collators.len() as u16 + PADDING);
                RowGroup {
                    first_row,
                    collators,
                }
            })
            .collect()
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    pub fn is_active(&self) -> bool {
        self.focus.is_active()
    }

    pub fn focus(&self) -> ListFocus {
        self.focus
    }

    /// Sets where the focus is. Back from the main window, where every collator can be selected,
    /// the selection moves to the nearest tracked one, the only ones the compact list has.
    pub fn set_focus(&mut self, focus: ListFocus) {
        let was_main = self.focus == ListFocus::Main;
        self.focus = focus;
        if was_main && focus != ListFocus::Main && !self.selected_collator_is_tracked() {
            let selected = self.table_state.selected();
            let len = self.collators_order.len();
            let nearest_tracked = selected.and_then(|sel| {
                (0..len)
                    .map(|step| (sel + step) % len)
                    .find(|i| self.is_tracked_at(*i))
            });
            self.table_state.select(nearest_tracked);
            self.ensure_selection_in_view();
        }
    }

    fn selected_collator_is_tracked(&self) -> bool {
        self.get_selected_ref().is_some_and(|c| c.is_tracked())
    }

    pub fn view(&self) -> CollatorsView {
        self.view
    }

    pub fn toggle_view(&mut self) {
        self.view = match self.view {
            CollatorsView::Group => CollatorsView::List,
            CollatorsView::List => CollatorsView::Group,
        };
    }

    pub fn is_masked(&self) -> bool {
        self.masked
    }

    pub fn toggle_mask(&mut self) {
        self.masked = !self.masked;
    }

    pub fn is_list_view(&self) -> bool {
        self.view == CollatorsView::List
    }

    pub fn get_selected(&self) -> Option<Collator> {
        self.table_state
            .selected()
            .and_then(|i| self.get_collator_by_index_cloned(i))
    }

    pub fn is_proxy_valid(&self) -> bool {
        if let Some(c) = self.get_selected() {
            return c.is_proxy_valid();
        }
        false
    }

    pub fn has_commands_available(&self) -> bool {
        if let Some(c) = self.get_selected() {
            return c.has_commands_available();
        }
        false
    }

    pub fn move_down(&mut self) -> Option<Collator> {
        self.move_selection(true)
    }

    pub fn move_up(&mut self) -> Option<Collator> {
        self.move_selection(false)
    }

    /// Moves the selection to the next (or previous) collator, wrapping around. Unless the
    /// main window is focused, untracked collators, the ones only found on-chain, are skipped
    /// since they are not listed in the compact view the selection is driven from.
    fn move_selection(&mut self, forward: bool) -> Option<Collator> {
        let len = self.collators_order.len();
        let can_select =
            |list: &Self, i: usize| list.focus == ListFocus::Main || list.is_tracked_at(i);

        // Nothing selected yet, e.g. no collator is tracked: start from the first one the
        // focus allows
        let first = (0..len).find(|i| can_select(self, *i))?;
        let selected = match self.table_state.selected() {
            Some(selected) => selected,
            None => {
                self.navigated = true;
                self.table_state.select(Some(first));
                self.ensure_selection_in_view();
                return self.get_collator_by_index_cloned(first);
            }
        };

        // The first time the selection is moved, start from the top instead of from wherever
        // the selection happens to be
        if !self.navigated {
            self.navigated = true;
            if selected != first {
                self.scroll_offset = 0;
                self.table_state.select(Some(first));
                self.ensure_selection_in_view();
                return self.get_collator_by_index_cloned(first);
            }
        }

        let next = (1..=len)
            .map(|step| {
                if forward {
                    (selected + step) % len
                } else {
                    (selected + len - step) % len
                }
            })
            .find(|i| can_select(self, *i))?;

        // Wrapped around to the beginning, start again from the top
        if forward && next <= selected {
            self.scroll_offset = 0;
        }
        self.table_state.select(Some(next));
        self.ensure_selection_in_view();
        self.get_collator_by_index_cloned(next)
    }

    fn is_tracked_at(&self, index: usize) -> bool {
        self.get_collator_by_index(index)
            .is_some_and(|c| c.is_tracked())
    }

    fn selected_key(&self) -> Option<CollatorKey> {
        self.table_state
            .selected()
            .and_then(|i| self.collators_order.get(i))
            .cloned()
    }

    /// Selects `key` again after the order changed, or the first tracked collator (the first one
    /// when none is tracked) when there is no `key` or it is gone.
    fn restore_selection(&mut self, key: Option<CollatorKey>) {
        let index = key
            .and_then(|key| self.collators_order.iter().position(|k| *k == key))
            .or_else(|| (0..self.collators_order.len()).find(|i| self.is_tracked_at(*i)))
            // None is tracked, e.g. only the on-chain collators are listed, so start from the
            // first one to be able to move from there on the main window
            .or_else(|| (!self.collators_order.is_empty()).then_some(0));
        self.table_state.select(index);
        // Only follow the selection while it is being used, otherwise keep showing the top
        if self.focus.is_active() {
            self.ensure_selection_in_view();
        } else {
            self.scroll_offset = 0;
        }
    }

    /// Marks the collators of `runtime` found in the current Aura authority set.
    ///
    /// Leaves collators already marked `Invulnerable` untouched, since that status
    /// takes priority regardless of the order the two fetches resolve in.
    ///
    /// With `is_all_collators_visible`, the authorities not known yet are added first, as on-chain only
    /// collators, so they get a status too.
    pub fn update_authorities(
        &mut self,
        runtime: SupportedRuntime,
        authorities: &[AccountBytes],
        is_all_collators_visible: bool,
    ) {
        if is_all_collators_visible {
            self.add_onchain_collators(runtime, authorities.iter().copied());
        }
        for collator in self.collators.values_mut() {
            if collator.runtime() == runtime && *collator.status() != CollatorStatus::Invulnerable {
                let stash_bytes: AccountBytes = *collator.stash().as_ref();
                if authorities.contains(&stash_bytes) {
                    if collator.deposit().is_zero() && collator.has_keys() {
                        collator.set_status(CollatorStatus::Exiting);
                    } else {
                        collator.set_status(CollatorStatus::Permissionless);
                    }
                }
            }
        }
    }

    /// Marks the collators of `runtime` found in the invulnerable set, overriding
    /// whatever status they currently have.
    ///
    /// With `is_all_collators_visible`, the invulnerables not known yet are added first, as on-chain only
    /// collators, so they get a status too.
    pub fn update_invulnerables(
        &mut self,
        runtime: SupportedRuntime,
        invulnerables: &[AccountBytes],
        is_all_collators_visible: bool,
    ) {
        if is_all_collators_visible {
            self.add_onchain_collators(runtime, invulnerables.iter().copied());
        }
        for collator in self.collators.values_mut() {
            if collator.runtime() == runtime {
                let stash_bytes: AccountBytes = *collator.stash().as_ref();
                if invulnerables.contains(&stash_bytes) {
                    collator.set_status(CollatorStatus::Invulnerable);
                }
            }
        }
    }

    /// Marks the collators of `runtime` found in the candidate list.
    ///
    /// Leaves collators already marked `Invulnerable` or `Permissionless` untouched, since those status
    /// takes priority regardless of the order the fetches resolve in.
    ///
    /// With `is_all_collators_visible`, the candidates not known yet are added first, as on-chain only
    /// collators, so they get a status too.
    pub fn update_candidates(
        &mut self,
        runtime: SupportedRuntime,
        candidates: &[(AccountBytes, Amount)],
        is_all_collators_visible: bool,
    ) {
        if is_all_collators_visible {
            self.add_onchain_collators(runtime, candidates.iter().map(|(who, _)| *who));
        }
        for collator in self.collators.values_mut() {
            if collator.runtime() == runtime {
                let stash_bytes: AccountBytes = *collator.stash().as_ref();
                if let Some((_, deposit)) = candidates.iter().find(|(who, _)| *who == stash_bytes) {
                    if *collator.status() == CollatorStatus::Unknown {
                        collator.set_status(CollatorStatus::Candidate);
                    }

                    collator.set_deposit(*deposit);
                }
            }
        }
    }

    /// Records `block_number` against whichever collator of `runtime` was assigned `slot`,
    /// per the Aura round-robin rule `authorities[slot % authorities.len()]`.
    pub fn update_authored_block(
        &mut self,
        runtime: SupportedRuntime,
        authorities: &[[u8; 32]],
        block_number: u64,
        slot: u64,
    ) {
        if authorities.is_empty() {
            return;
        }
        let author_index = (slot % authorities.len() as u64) as usize;
        let Some(author_bytes) = authorities.get(author_index) else {
            return;
        };
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        for collator in self.collators.values_mut() {
            if collator.runtime() == runtime {
                let stash_bytes: [u8; 32] = *collator.stash().as_ref();
                if stash_bytes == *author_bytes {
                    collator.set_last_block_authored(block_number, ts);
                    collator.increment_block_in_slot(slot);
                }
            }
        }
    }

    /// Seeds the collator's last produced block number, the `current_block`/`block_time_ms`
    /// are used to estimate how long ago that block was authored.
    pub fn update_last_authored_block(
        &mut self,
        collator_key: &CollatorKey,
        block_number: u64,
        current_block: u64,
        block_time_ms: Option<u64>,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let ts = match block_time_ms {
            Some(block_time_ms) => {
                let blocks_ago = current_block.saturating_sub(block_number);
                now.saturating_sub(blocks_ago as u128 * block_time_ms as u128)
            }
            None => now,
        };

        if let Some(collator) = self.collators.get_mut(collator_key) {
            if block_number > collator.last_block_authored().unwrap_or(0) {
                collator.set_last_block_authored(block_number, ts);
            }
        }
    }

    /// Sets the on-chain identity for the collator matching `collator_key`.
    /// Sets the on-chain identity for every collator matching `stash_bytes`, regardless
    /// of runtime: identity is fetched once per stash via the People chain and fans out
    /// to every chain where that same stash runs as a collator.
    pub fn update_identity(&mut self, stash_bytes: [u8; 32], identity: Identity) {
        for collator in self.collators.values_mut() {
            let collator_stash: [u8; 32] = *collator.stash().as_ref();
            if collator_stash == stash_bytes {
                collator.set_identity(Some(identity.clone()));
            }
        }
    }

    pub fn update_next_keys(&mut self, collator_key: &CollatorKey, keys: Option<[u8; 32]>) {
        if let Some(collator) = self.collators.get_mut(collator_key) {
            collator.set_next_keys(keys);
        }
    }

    pub fn update_queued_keys(&mut self, collator_key: &CollatorKey, keys: Option<[u8; 32]>) {
        if let Some(collator) = self.collators.get_mut(collator_key) {
            collator.set_queued_keys(keys);
        }
    }

    pub fn add_proxy(&mut self, collator_key: &CollatorKey, proxy: ProxyKey) {
        if let Some(collator) = self.collators.get_mut(collator_key) {
            collator.proxies.insert(proxy);
        }
    }

    pub fn update_status(&mut self, collator_key: &CollatorKey, status: CollatorStatus) {
        if let Some(collator) = self.collators.get_mut(collator_key) {
            collator.set_status(status);
        }
    }

    fn set_balance(&mut self, collator_key: &AccountKey, balance: Balance) {
        if let Some(collator) = self.collators.get_mut(collator_key) {
            collator.account.set_balance(balance);
        }
    }

    pub fn update_balance(&mut self, collator_key: &CollatorKey, balance: Balance) {
        self.set_balance(collator_key, balance);
    }

    fn set_deposit(&mut self, collator_key: &AccountKey, amount: Amount) {
        if let Some(collator) = self.collators.get_mut(collator_key) {
            collator.set_deposit(amount);
        }
    }

    pub fn update_deposit(&mut self, collator_key: &CollatorKey, amount: Amount) {
        self.set_deposit(collator_key, amount);
    }
}
