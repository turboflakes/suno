use crate::widgets::collators_detailed_group::{GROUP_HEADER_HEIGHT, PADDING};
use ratatui::widgets::TableState;
use std::{
    collections::BTreeMap,
    time::{SystemTime, UNIX_EPOCH},
};
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
use suno_primitives::{
    collator::{Collator, CollatorStatus},
    identity::Identity,
    AccountDisplay, AccountKey,
};

#[derive(Debug, Default)]
pub struct CollatorsList {
    pub collators: Vec<Collator>,
    pub table_state: TableState,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    active: bool,
}

impl CollatorsList {
    pub fn on_init(&mut self) {
        let chains = CONFIG.chains();
        for chain in chains.iter() {
            for (chain_name, chain_config) in chain {
                for collator in &chain_config.collators {
                    match collator {
                        NodeConfig::Address(stash) => {
                            self.collators.push(Collator::new(*chain_name, *stash));
                        }
                        NodeConfig::Detailed { stash, .. } => {
                            self.collators.push(Collator::new(*chain_name, *stash));
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
        self.collators.iter()
    }

    pub fn get_collator_keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        self.collators
            .iter()
            .filter(|c| c.runtime() == runtime)
            .map(|c| c.key().clone())
            .collect()
    }

    pub fn get_collators_grouped_by_runtime(&self) -> BTreeMap<SupportedRuntime, Vec<&Collator>> {
        let mut grouped: BTreeMap<SupportedRuntime, Vec<&Collator>> = BTreeMap::new();

        for collator in &self.collators {
            grouped
                .entry(collator.runtime())
                .or_default()
                .push(collator);
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
            .and_then(|i| self.collators.get(i))
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

    pub fn get_selected(&self) -> Option<Collator> {
        self.table_state
            .selected()
            .and_then(|i| self.collators.get(i).cloned())
    }

    pub fn move_down(&mut self) -> Option<Collator> {
        if let Some(selected) = self.table_state.selected() {
            if selected == self.collators.len() - 1 {
                self.table_state.select_first();
                self.scroll_offset = 0;
            } else {
                self.table_state.scroll_down_by(1);
            }
            self.ensure_selection_in_view();
            self.table_state
                .selected()
                .and_then(|i| self.collators.get(i).cloned())
        } else {
            None
        }
    }

    pub fn move_up(&mut self) -> Option<Collator> {
        if let Some(selected) = self.table_state.selected() {
            if selected == 0 {
                let i = self.collators.len() - 1;
                self.table_state.select(Some(i));
            } else {
                self.table_state.scroll_up_by(1);
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            self.ensure_selection_in_view();
            self.table_state
                .selected()
                .and_then(|i| self.collators.get(i).cloned())
        } else {
            None
        }
    }

    /// Marks the collators of `runtime` found in the current Aura authority set.
    ///
    /// Leaves collators already marked `Invulnerable` untouched, since that status
    /// takes priority regardless of the order the two fetches resolve in.
    pub fn update_aura_authorities(&mut self, runtime: SupportedRuntime, authorities: &[[u8; 32]]) {
        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime && *collator.status() != CollatorStatus::Invulnerable {
                let stash_bytes: [u8; 32] = *collator.stash().as_ref();
                let status = if authorities.contains(&stash_bytes) {
                    CollatorStatus::Authority
                } else {
                    CollatorStatus::Unknown
                };
                collator.set_status(status);
            }
        }
    }

    /// Marks the collators of `runtime` found in the invulnerable set, overriding
    /// whatever status they currently have.
    pub fn update_invulnerables(&mut self, runtime: SupportedRuntime, invulnerables: &[[u8; 32]]) {
        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime {
                let stash_bytes: [u8; 32] = *collator.stash().as_ref();
                if invulnerables.contains(&stash_bytes) {
                    collator.set_status(CollatorStatus::Invulnerable);
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

        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime {
                let stash_bytes: [u8; 32] = *collator.stash().as_ref();
                if stash_bytes == *author_bytes {
                    collator.set_last_block_authored(block_number, ts);
                    collator.increment_block_in_slot(slot);
                }
            }
        }
    }

    /// Seeds collators last produced block number, the `current_block`/`block_time_ms`
    /// are used to estimate how long ago that block was authored.
    pub fn update_last_authored_block(
        &mut self,
        runtime: SupportedRuntime,
        stash_bytes: [u8; 32],
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

        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime {
                let collator_stash: [u8; 32] = *collator.stash().as_ref();
                if collator_stash == stash_bytes
                    && block_number > collator.last_block_authored().unwrap_or(0)
                {
                    collator.set_last_block_authored(block_number, ts);
                }
            }
        }
    }

    /// Sets the on-chain identity for whichever collator of `runtime` matches `stash_bytes`.
    pub fn update_identity(
        &mut self,
        runtime: SupportedRuntime,
        stash_bytes: [u8; 32],
        identity: Identity,
    ) {
        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime {
                let collator_stash: [u8; 32] = *collator.stash().as_ref();
                if collator_stash == stash_bytes {
                    collator.set_identity(Some(identity.clone()));
                }
            }
        }
    }

    pub fn update_next_keys(
        &mut self,
        runtime: SupportedRuntime,
        stash_bytes: [u8; 32],
        keys: Option<[u8; 32]>,
    ) {
        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime {
                let collator_stash: [u8; 32] = *collator.stash().as_ref();
                if collator_stash == stash_bytes {
                    collator.set_next_keys(keys);
                }
            }
        }
    }

    pub fn update_queued_keys(
        &mut self,
        runtime: SupportedRuntime,
        stash_bytes: [u8; 32],
        keys: Option<[u8; 32]>,
    ) {
        for collator in self.collators.iter_mut() {
            if collator.runtime() == runtime {
                let collator_stash: [u8; 32] = *collator.stash().as_ref();
                if collator_stash == stash_bytes {
                    collator.set_queued_keys(keys);
                }
            }
        }
    }
}
