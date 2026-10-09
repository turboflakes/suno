use crate::widgets::collators_detailed_group::{GROUP_HEADER_HEIGHT, PADDING};
use ratatui::widgets::TableState;
use sp_arithmetic::traits::Zero;
use std::{
    collections::{BTreeMap, HashMap},
    time::{SystemTime, UNIX_EPOCH},
};
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

#[derive(Debug, Default)]
pub struct CollatorsList {
    pub collators: HashMap<CollatorKey, Collator>,
    pub collators_order: Vec<CollatorKey>,
    pub table_state: TableState,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    active: bool,
    view: CollatorsView,
}

impl CollatorsList {
    pub fn add_collator(&mut self, collator: Collator) {
        let key = collator.key().clone();
        if !self.collators.contains_key(&key) {
            self.collators_order.push(key.clone());
        }
        self.collators.insert(key, collator);
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
                        NodeConfig::Detailed { stash, .. } => {
                            self.add_collator(Collator::new(*chain_name, *stash));
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
        let selected_y_position = self.get_selected_y_position();

        if selected_y_position < self.scroll_offset {
            self.scroll_offset = selected_y_position;
        } else if selected_y_position >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = selected_y_position - self.viewport_height + 1;
        }
    }

    // Determine the Y position of the current collator selection
    fn get_selected_y_position(&self) -> u16 {
        let mut selected_y_position = 0;
        let selected_ref = self.get_selected_ref();

        for (_, collators) in self.get_collators_grouped_by_runtime() {
            if let Some(idx) = collators.iter().position(|c| Some(*c) == selected_ref) {
                // Header + index + table header
                return selected_y_position + GROUP_HEADER_HEIGHT + idx as u16 + 1;
            }
            selected_y_position += GROUP_HEADER_HEIGHT + collators.len() as u16 + PADDING;
        }
        0
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
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
        if let Some(selected) = self.table_state.selected() {
            if selected == self.collators_order.len() - 1 {
                self.table_state.select_first();
                self.scroll_offset = 0;
            } else {
                self.table_state.scroll_down_by(1);
            }
            self.ensure_selection_in_view();
            self.table_state
                .selected()
                .and_then(|i| self.get_collator_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn move_up(&mut self) -> Option<Collator> {
        if let Some(selected) = self.table_state.selected() {
            if selected == 0 {
                let i = self.collators_order.len() - 1;
                self.table_state.select(Some(i));
            } else {
                self.table_state.scroll_up_by(1);
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            self.ensure_selection_in_view();
            self.table_state
                .selected()
                .and_then(|i| self.get_collator_by_index_cloned(i))
        } else {
            None
        }
    }

    /// Marks the collators of `runtime` found in the current Aura authority set.
    ///
    /// Leaves collators already marked `Invulnerable` untouched, since that status
    /// takes priority regardless of the order the two fetches resolve in.
    pub fn update_aura_authorities(
        &mut self,
        runtime: SupportedRuntime,
        authorities: &[AccountBytes],
    ) {
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
    pub fn update_invulnerables(
        &mut self,
        runtime: SupportedRuntime,
        invulnerables: &[AccountBytes],
    ) {
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
    pub fn update_candidates(
        &mut self,
        runtime: SupportedRuntime,
        candidates: &[(AccountBytes, Amount)],
    ) {
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
