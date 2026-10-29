use crate::widgets::list_focus::{ListFocus, VISIBLE_ROWS_MARGIN};
use crate::widgets::validators_detailed_group::{GROUP_HEADER_HEIGHT, PADDING};
use crate::widgets::{input_field::InputFieldWidget, input_filter::matches_query};
use ratatui::widgets::TableState;
use std::collections::{BTreeMap, HashMap, HashSet};
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::utils::AccountId32;
use suno_config::{fetch_validators_from_source, NodeConfig, PinnedNode, SupportedRuntime, CONFIG};
use suno_primitives::{
    balance::Balance,
    identity::Identity,
    origin::Origin,
    proxy::ProxyKey,
    session::Keys,
    staking::{Chunk, Payee, StakeLedger, StakeOverview, ValidatorPrefs},
    validator::{Validator, ValidatorStatus},
    AccountKey,
};
use tracing::error;

type Points = u32;
type Amount = u128;
type ValidatorKey = AccountKey;
type AccountBytes = [u8; 32];

/// A group of validators in the detailed group view, and the row of its first validator.
struct RowGroup<'a> {
    first_row: u16,
    validators: Vec<&'a Validator>,
}

#[derive(Debug)]
pub struct ValidatorsList {
    pub validators: HashMap<ValidatorKey, Validator>,
    pub validators_order: Vec<ValidatorKey>,
    pub table_state: TableState,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    focus: ListFocus,
    masked: bool,
    /// Whether the selection was moved by the user already. Until then, the selected row is
    /// whatever the list started with.
    navigated: bool,
    /// Validators only found on-chain whose data was requested already, since it is only
    /// fetched for the ones displayed.
    requested: HashSet<ValidatorKey>,
    /// Validators whose identity was requested already
    identity_requested: HashSet<ValidatorKey>,
    /// Whether the window with this list is the one displayed.
    visible: bool,
    /// Filters the validators listed in the detailed view
    filter: InputFieldWidget,
    /// What is typed in the filter, in lowercase
    query: String,
}

impl Default for ValidatorsList {
    fn default() -> Self {
        Self {
            validators: HashMap::new(),
            validators_order: Vec::new(),
            table_state: TableState::default(),
            scroll_offset: 0,
            viewport_height: 0,
            focus: ListFocus::default(),
            masked: true,
            navigated: false,
            requested: HashSet::new(),
            identity_requested: HashSet::new(),
            visible: false,
            filter: InputFieldWidget::new_filter(),
            query: String::new(),
        }
    }
}

impl ValidatorsList {
    pub fn add_validator(&mut self, validator: Validator) {
        let key = validator.key();
        if !self.validators.contains_key(key) {
            // Keep the order sorted by runtime (insertion order within a runtime), so the
            // validators of a runtime are listed together and the selection order matches the
            // order of the groups displayed.
            let position = self
                .validators_order
                .partition_point(|k| k.runtime <= key.runtime);
            self.validators_order.insert(position, key.clone());
        }
        self.validators.insert(key.clone(), validator);
    }

    /// Updates every validator of `runtime`, the relay chain, active or waiting. The validators
    /// not known yet are added as on-chain only ones, and the on-chain only ones that are no
    /// longer in the set are removed. Returns the keys of the validators added.
    pub fn update_all_validators(
        &mut self,
        runtime: SupportedRuntime,
        stashes: &[AccountBytes],
    ) -> Vec<ValidatorKey> {
        // An empty set is not a valid set of validators, so nothing changes
        if stashes.is_empty() {
            return Vec::new();
        }

        let selected = self.selected_key();
        let mut added = Vec::new();
        for bytes in stashes {
            let key = ValidatorKey::from_bytes(runtime, *bytes);
            if self.validators.contains_key(&key) {
                continue;
            }
            self.add_validator(Validator::with_origin(
                runtime,
                AccountId32::from(*bytes),
                Origin::OnChain,
            ));
            added.push(key);
        }

        let keep: HashSet<&AccountBytes> = stashes.iter().collect();
        let stale: Vec<ValidatorKey> = self
            .validators
            .iter()
            .filter(|(key, v)| {
                !v.is_tracked() && key.runtime == runtime && !keep.contains(&key.bytes)
            })
            .map(|(key, _)| key.clone())
            .collect();
        for key in &stale {
            self.validators.remove(key);
            self.requested.remove(key);
            self.identity_requested.remove(key);
        }
        self.validators_order.retain(|key| !stale.contains(key));

        // New validators are inserted in order and others removed, which shifts the selected index
        let selected = selected.filter(|key| !stale.contains(key));
        if !added.is_empty() || !stale.is_empty() {
            self.restore_selection(selected);
        }
        added
    }

    pub fn set_prefs(&mut self, validator_key: &AccountKey, prefs: ValidatorPrefs) -> bool {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            if validator.prefs != prefs {
                validator.prefs = prefs;
                return true;
            }
        }
        false
    }

    pub fn set_prefs_next(&mut self, validator_key: &AccountKey, prefs: ValidatorPrefs) -> bool {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            if validator.prefs_next != prefs {
                validator.prefs_next = prefs;
                return true;
            }
        }
        false
    }

    pub fn set_points(&mut self, validator_key: &AccountKey, points: Points) -> bool {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            if validator.points != points {
                let old_points = validator.points;
                validator.points = points;
                validator.old_points = old_points;
                let ts = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_millis();
                validator.old_points_ts = ts;
                return true;
            }
        }
        false
    }

    pub fn set_era_points(&mut self, validator_key: &AccountKey, points: Points) -> bool {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            if validator.era_points != points {
                validator.era_points = points;
                return true;
            }
        }
        false
    }

    pub fn set_identity(&mut self, validator_key: &AccountKey, identity: Identity) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.account.set_identity(Some(identity));
        }
    }

    pub fn set_stake_overview(&mut self, validator_key: &AccountKey, data: StakeOverview) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.stake = data;
        }
    }

    pub fn set_stake_ledger(&mut self, validator_key: &AccountKey, data: StakeLedger) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.ledger = data;

            // NOTE: If the ledger is updated and the validator status is unknowe
            // the validator status should be changed to waiting. Meaning that valitor_prefs
            // should become available
            if validator.status == ValidatorStatus::Unknown {
                validator.status = ValidatorStatus::Waiting;
            }
        }
    }

    pub fn set_payee(&mut self, validator_key: &AccountKey, data: Payee) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.payee = data;
        }
    }

    pub fn set_next_keys(&mut self, validator_key: &AccountKey, data: Option<Keys>) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.next_keys = data;
        }
    }

    pub fn set_queued_keys(&mut self, validator_key: &AccountKey, data: Option<Keys>) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.queued_keys = data;
        }
    }

    pub fn add_amount_to_stake_ledger(&mut self, validator_key: &AccountKey, amount: Amount) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.ledger = StakeLedger {
                active: validator.ledger.active().saturating_add(amount),
                total: validator.ledger.total().saturating_add(amount),
                unlocking: validator.ledger.unlocking().to_vec(),
            };
        }
    }

    pub fn sub_chunk_from_stake_ledger(&mut self, validator_key: &AccountKey, chunk: Chunk) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            let unlocking: Vec<Chunk> = if validator.ledger.unlocking().is_empty() {
                vec![chunk.clone()]
            } else {
                validator
                    .ledger
                    .unlocking()
                    .iter()
                    .map(|c| {
                        if c.era == chunk.era {
                            Chunk {
                                era: c.era,
                                value: c.value.saturating_add(chunk.value),
                            }
                        } else {
                            c.clone()
                        }
                    })
                    .collect()
            };

            validator.ledger = StakeLedger {
                active: validator.ledger.active().saturating_sub(chunk.value),
                total: validator.ledger.total().saturating_sub(chunk.value),
                unlocking,
            };
        }
    }

    pub fn set_status(&mut self, validator_key: &AccountKey, status: ValidatorStatus) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.status = status;
        }
    }

    pub fn set_proxies(&mut self, validator_key: &AccountKey, proxies: HashSet<ProxyKey>) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.proxies = proxies;
        }
    }

    pub fn add_proxy(&mut self, validator_key: &AccountKey, proxy: ProxyKey) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.proxies.insert(proxy);
        }
    }

    pub fn set_balance(&mut self, validator_key: &AccountKey, balance: Balance) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.account.set_balance(balance);
        }
    }

    pub fn add_amount_to_balance(&mut self, validator_key: &AccountKey, amount: Amount) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.account.add_free_amount(amount);
        }
    }

    pub fn set_viewport_height(&mut self, height: u16) {
        self.viewport_height = height;
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

    /// Whether a row is highlighted, which it isn't while the filter is being typed.
    pub fn is_highlighted(&self) -> bool {
        self.focus.is_highlighted()
    }

    pub fn focus(&self) -> ListFocus {
        self.focus
    }

    pub fn is_masked(&self) -> bool {
        self.masked
    }

    pub fn get_validator_by_key(&self, validator_key: &ValidatorKey) -> Option<&Validator> {
        self.validators.get(validator_key)
    }

    pub fn get_validator_by_key_cloned(&self, validator_key: &ValidatorKey) -> Option<Validator> {
        self.validators.get(validator_key).cloned()
    }

    // Helper method to get validator by table index
    pub fn get_validator_by_index(&self, index: usize) -> Option<&Validator> {
        self.validators_order
            .get(index)
            .and_then(|key| self.validators.get(key))
    }

    pub fn get_validator_by_index_cloned(&self, index: usize) -> Option<Validator> {
        self.get_validator_by_index(index).cloned()
    }

    /// Returns an iterator of validators in display order
    pub fn validators_iter(&self) -> impl Iterator<Item = &Validator> {
        self.validators_order
            .iter()
            .filter_map(move |key| self.validators.get(key))
    }

    /// Returns true if any validator has proxies available
    pub fn proxies_available(&self) -> bool {
        self.validators_order
            .iter()
            .any(|key| self.validators.get(key).is_some_and(|v| v.has_proxies()))
    }

    /// Get all AccountKeys for a specific runtime
    pub fn get_keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        self.validators_order
            .iter()
            .filter(|key| key.runtime == runtime)
            .cloned()
            .collect()
    }

    pub fn get_keys_grouped_by_runtime_cloned(&self) -> HashMap<SupportedRuntime, Vec<AccountKey>> {
        let mut grouped: HashMap<SupportedRuntime, Vec<AccountKey>> = HashMap::new();

        for key in self.validators.keys() {
            grouped.entry(key.runtime).or_default().push(key.clone());
        }

        grouped
    }

    pub fn get_validators_grouped_by_runtime(&self) -> BTreeMap<SupportedRuntime, Vec<&Validator>> {
        let mut grouped: BTreeMap<SupportedRuntime, Vec<&Validator>> = BTreeMap::new();

        for key in &self.validators_order {
            if let Some(validator) = self.get_validator_by_key(key) {
                if self.matches_filter(validator) {
                    grouped.entry(key.runtime).or_default().push(validator);
                }
            }
        }

        grouped
    }

    pub fn filter(&self) -> &InputFieldWidget {
        &self.filter
    }

    /// Edits the filter. When what is typed changes, the selection goes to the first validator
    /// that still matches.
    pub fn edit_filter(&mut self, edit: impl FnOnce(&mut InputFieldWidget)) {
        edit(&mut self.filter);
        let query = self.filter.raw_value().to_lowercase();
        if query != self.query {
            self.query = query;
            self.select_first_match();
        }
    }

    /// Whether something is typed in the filter.
    pub fn is_filtered(&self) -> bool {
        !self.query.is_empty()
    }

    /// How many validators match the filter.
    pub fn count_matches(&self) -> usize {
        self.validators_iter()
            .filter(|v| self.matches_filter(v))
            .count()
    }

    fn matches_filter(&self, validator: &Validator) -> bool {
        matches_query(
            &self.query,
            &[&validator.display_identity(), validator.account.address()],
        )
    }

    fn matches_filter_at(&self, index: usize) -> bool {
        self.get_validator_by_index(index)
            .is_some_and(|v| self.matches_filter(v))
    }

    /// Selects the first validator that can be selected and matches the filter, from the top.
    pub fn select_first_match(&mut self) {
        let first = (0..self.validators_order.len()).find(|i| {
            (self.focus.selects_every_row() || self.is_tracked_at(*i)) && self.matches_filter_at(*i)
        });
        self.table_state.select(first);
        self.scroll_offset = 0;
    }

    /// Height needed by [`ValidatorsCompactWidget`] to display every validator without scrolling.
    /// One row per validator and one extra row per distinct runtime group, and at least one row
    /// for the empty set message.
    pub fn total_compact_height(&self) -> u16 {
        let tracked: Vec<&Validator> = self.validators_iter().filter(|v| v.is_tracked()).collect();
        let runtimes: HashSet<SupportedRuntime> = tracked.iter().map(|v| v.runtime()).collect();
        let rows = (tracked.len() + runtimes.len()).max(1);
        3 + rows as u16
    }

    /// The selected validator, only while its row is highlighted.
    pub fn get_highlighted_ref(&self) -> Option<&Validator> {
        self.is_highlighted().then(|| self.get_selected_ref())?
    }

    pub fn get_selected_ref(&self) -> Option<&Validator> {
        self.table_state
            .selected()
            .and_then(|i| self.get_validator_by_index(i))
    }

    pub fn get_selected(&self) -> Option<Validator> {
        self.table_state
            .selected()
            .and_then(|i| self.get_validator_by_index_cloned(i))
    }

    // Helpers specific to the ValidatorsDetailedGroupWidget

    pub fn total_detailed_group_height(&self) -> u16 {
        let validators_grouped = self.get_validators_grouped_by_runtime();

        validators_grouped
            .values()
            .map(|v| GROUP_HEADER_HEIGHT + v.len() as u16 + PADDING)
            .sum()
    }

    // Scroll to the selected validator if it's not in view
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

    // Determine the Y position of the current validator selection
    fn get_selected_y_position(&self) -> u16 {
        let selected_ref = self.get_selected_ref();

        self.get_row_groups()
            .iter()
            .find_map(|group| {
                let idx = group
                    .validators
                    .iter()
                    .position(|v| Some(*v) == selected_ref)?;
                Some(group.first_row + idx as u16)
            })
            .unwrap_or(0)
    }

    /// The groups of validators as laid out in the detailed group view, by relay chain.
    fn get_row_groups(&self) -> Vec<RowGroup<'_>> {
        let grouped = self.get_validators_grouped_by_runtime();
        // Groups get a one row top padding when more than one is displayed
        let top_padding = if grouped.len() > 1 { 1 } else { 0 };
        let mut group_y = 0u16;
        grouped
            .into_values()
            .map(|validators| {
                // Top padding + header + table header
                let first_row = group_y + top_padding + GROUP_HEADER_HEIGHT + 1;
                group_y =
                    group_y.saturating_add(GROUP_HEADER_HEIGHT + validators.len() as u16 + PADDING);
                RowGroup {
                    first_row,
                    validators,
                }
            })
            .collect()
    }

    // Scroll down if content is taller than the screen
    pub fn scroll_down(&mut self, viewport_height: u16, total_content_height: u16) {
        if total_content_height > viewport_height {
            let max_scroll = total_content_height.saturating_sub(viewport_height);
            self.scroll_offset = (self.scroll_offset + 1).min(max_scroll);
        }
    }

    pub fn scroll_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(1);
    }

    pub async fn on_init(&mut self) {
        if !CONFIG.features().validators_enabled() {
            return;
        }

        let chains = CONFIG.chains();
        for chain in chains.iter() {
            for (chain_name, chain_config) in chain {
                // Process validators from the chain config.
                self.process_validators(chain_name, &chain_config.validators, Origin::Config);
                // Fetch validators from the external source, if defined.
                if let Some(source) = &chain_config.validators_source {
                    let res = fetch_validators_from_source(source).await;
                    match res {
                        Ok(validators) => {
                            self.process_validators(chain_name, &validators, Origin::Remote);
                        }
                        Err(e) => error!("{:?}", e),
                    }
                }
            }
        }

        // The validators pinned from the TUI, of the chains configured, unless already listed
        let configured: HashSet<SupportedRuntime> = chains
            .iter()
            .flat_map(|chain| chain.keys().copied())
            .collect();
        for pinned in CONFIG.pinned_validators() {
            let validator = Validator::with_origin(pinned.chain, pinned.stash, Origin::Pinned);
            if configured.contains(&pinned.chain) && !self.validators.contains_key(validator.key())
            {
                self.add_validator(validator);
            }
        }

        self.init_table();
    }

    /// Pins the validator when it was only found on-chain, and the other way
    /// around. The validators listed by the config are always tracked, so they stay as they are.
    /// Returns the new origin of the validator, if it changed.
    pub fn toggle_pinned(&mut self, key: &ValidatorKey) -> Option<Origin> {
        let validator = self.validators.get_mut(key)?;
        let origin = validator.origin.toggle()?;
        validator.origin = origin;

        // The selection can only be on a tracked validator, outside of the main window
        if !self.focus.selects_every_row() {
            self.select_nearest_tracked();
        }
        Some(origin)
    }

    /// The validators pinned from the TUI, to be persisted.
    pub fn get_pinned(&self) -> Vec<PinnedNode> {
        self.validators_iter()
            .filter(|v| v.origin.is_pinned())
            .map(|v| PinnedNode {
                chain: v.runtime(),
                stash: v.account.stash(),
            })
            .collect()
    }

    /// Removes the validator from the list, e.g. when it is no longer pinned and the validators
    /// only found on-chain are not listed.
    pub fn remove(&mut self, key: &ValidatorKey) {
        let selected = self.selected_key().filter(|selected| selected != key);
        self.validators.remove(key);
        self.requested.remove(key);
        self.identity_requested.remove(key);
        self.validators_order.retain(|k| k != key);
        self.restore_selection(selected);
    }

    /// Removes the validators only found on-chain, e.g. when they are no longer listed.
    pub fn remove_on_chain(&mut self) {
        let keys: Vec<_> = self
            .validators_iter()
            .filter(|v| v.origin.is_onchain())
            .map(|v| v.key().clone())
            .collect();
        for key in keys {
            self.remove(&key);
        }
    }

    /// Moves the selection to the nearest tracked validator, the only ones the compact list has.
    fn select_nearest_tracked(&mut self) {
        if self.get_selected_ref().is_some_and(|v| v.is_tracked()) {
            return;
        }
        let len = self.validators_order.len();
        let nearest_tracked = self.table_state.selected().and_then(|sel| {
            (0..len)
                .map(|step| (sel + step) % len)
                .find(|i| self.is_tracked_at(*i))
        });
        self.table_state.select(nearest_tracked);
        self.ensure_selection_in_view();
    }

    fn process_validators(
        &mut self,
        chain_name: &SupportedRuntime,
        validators: &[NodeConfig],
        origin: Origin,
    ) {
        for validator in validators {
            match validator {
                NodeConfig::Address(stash) => {
                    let validator = Validator::with_origin(*chain_name, *stash, origin);
                    self.add_validator(validator);
                }
                NodeConfig::Detailed {
                    stash,
                    host_rpc,
                    ssh,
                    commands,
                    ..
                } => {
                    let mut validator = Validator::with_origin(*chain_name, *stash, origin);
                    validator.host_rpc = host_rpc.clone();
                    validator.ssh = ssh.clone();
                    if let Some(cmds) = commands {
                        validator.commands = cmds.clone();
                    }
                    self.add_validator(validator);
                }
            }
        }
    }

    pub fn move_down(&mut self) -> Option<Validator> {
        self.move_selection(true)
    }

    pub fn move_up(&mut self) -> Option<Validator> {
        self.move_selection(false)
    }

    /// Moves the selection to the next (or previous) validator, wrapping around. Unless the
    /// main window is focused, untracked validators, the ones only found on-chain, are skipped
    /// since they are not listed in the compact view the selection is driven from.
    fn move_selection(&mut self, forward: bool) -> Option<Validator> {
        let len = self.validators_order.len();
        let can_select = |list: &Self, i: usize| {
            (list.focus.selects_every_row() || list.is_tracked_at(i)) && list.matches_filter_at(i)
        };

        // Nothing selected yet, e.g. no validator is tracked: start from the first one the
        // focus allows
        let first = (0..len).find(|i| can_select(self, *i))?;
        let selected = match self.table_state.selected() {
            Some(selected) => selected,
            None => {
                self.navigated = true;
                self.table_state.select(Some(first));
                self.ensure_selection_in_view();
                return self.get_validator_by_index_cloned(first);
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
                return self.get_validator_by_index_cloned(first);
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
        self.get_validator_by_index_cloned(next)
    }

    fn is_tracked_at(&self, index: usize) -> bool {
        self.get_validator_by_index(index)
            .is_some_and(|v| v.is_tracked())
    }

    fn selected_key(&self) -> Option<ValidatorKey> {
        self.table_state
            .selected()
            .and_then(|i| self.validators_order.get(i))
            .cloned()
    }

    /// Selects `key` again after the order changed, or the first tracked validator (the first
    /// one when none is tracked) when there is no `key` or it is gone.
    fn restore_selection(&mut self, key: Option<ValidatorKey>) {
        let index = key
            .and_then(|key| self.validators_order.iter().position(|k| *k == key))
            .or_else(|| (0..self.validators_order.len()).find(|i| self.is_tracked_at(*i)))
            .or_else(|| (!self.validators_order.is_empty()).then_some(0));
        self.table_state.select(index);
        // Only follow the selection while it is being used, otherwise keep showing the top
        if self.focus.is_active() {
            self.ensure_selection_in_view();
        } else {
            self.scroll_offset = 0;
        }
    }

    pub fn init_table(&mut self) {
        if !self.validators.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    /// Sets where the focus is. Back from the main window, where every validator can be selected,
    /// the selection moves to the nearest tracked one, the only ones the compact list has.
    pub fn set_focus(&mut self, focus: ListFocus) {
        let was_every_row = self.focus.selects_every_row();
        self.focus = focus;
        if was_every_row && !focus.selects_every_row() {
            self.select_nearest_tracked();
        }
    }

    pub fn toggle_mask(&mut self) {
        self.masked = !self.is_masked();
    }

    pub fn is_proxy_valid(&self) -> bool {
        if let Some(v) = self.get_selected() {
            return v.is_proxy_valid();
        }
        false
    }

    pub fn has_commands_available(&self) -> bool {
        if let Some(v) = self.get_selected() {
            return v.has_commands_available();
        }
        false
    }

    pub fn get_validator_keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        self.get_keys_by_runtime(runtime.relay_chain())
    }

    /// Keys of the validators with a row in the viewport of the detailed group view, plus a page
    /// of margin above and below, so the data is there before the row is scrolled to.
    fn get_visible_keys(&self) -> Vec<ValidatorKey> {
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
            for (i, validator) in group.validators.iter().enumerate() {
                let row = group.first_row.saturating_add(i as u16);
                if row >= top && row < bottom {
                    keys.push(validator.key().clone());
                }
            }
        }
        keys
    }

    /// The validators only found on-chain that are visible and whose data wasn't requested
    /// yet, by runtime. They are marked as requested, so they are returned once.
    pub fn take_unrequested_visible(&mut self) -> HashMap<SupportedRuntime, Vec<ValidatorKey>> {
        let mut by_runtime: HashMap<SupportedRuntime, Vec<ValidatorKey>> = HashMap::new();
        for key in self.get_visible_keys() {
            let is_on_chain = self.validators.get(&key).is_some_and(|v| !v.is_tracked());
            if is_on_chain && self.requested.insert(key.clone()) {
                by_runtime.entry(key.runtime).or_default().push(key);
            }
        }
        by_runtime
    }

    /// The validators whose identity wasn't requested yet, by relay chain. The identity is
    /// fetched for every validator, listed or not, since it is what identifies them.
    pub fn get_keys_without_identity_requested(
        &self,
    ) -> HashMap<SupportedRuntime, Vec<ValidatorKey>> {
        let mut by_runtime: HashMap<SupportedRuntime, Vec<ValidatorKey>> = HashMap::new();
        for validator in self.validators_iter() {
            if validator.identity().is_none() && !self.identity_requested.contains(validator.key())
            {
                by_runtime
                    .entry(validator.runtime())
                    .or_default()
                    .push(validator.key().clone());
            }
        }
        by_runtime
    }

    pub fn mark_identity_requested(&mut self, keys: &[ValidatorKey]) {
        self.identity_requested.extend(keys.iter().cloned());
    }

    /// Keys to sync periodically: the tracked validators and the on-chain only ones visible.
    pub fn get_tracked_and_visible_keys_by_runtime(
        &self,
        runtime: SupportedRuntime,
    ) -> Vec<ValidatorKey> {
        let visible = self.get_visible_keys();
        self.get_validator_keys_by_runtime(runtime)
            .into_iter()
            .filter(|key| {
                self.validators.get(key).is_some_and(|v| v.is_tracked()) || visible.contains(key)
            })
            .collect()
    }

    /// Same as [`Self::get_validator_keys_by_runtime`], only for the tracked validators. The
    /// data that takes a fetch per validator is only fetched for these, not for the ones only
    /// found on-chain.
    pub fn get_tracked_validator_keys_by_runtime(
        &self,
        runtime: SupportedRuntime,
    ) -> Vec<AccountKey> {
        self.get_validator_keys_by_runtime(runtime)
            .into_iter()
            .filter(|key| self.validators.get(key).is_some_and(|v| v.is_tracked()))
            .collect()
    }

    pub fn update_prefs(&mut self, validator_key: &AccountKey, prefs: ValidatorPrefs) -> bool {
        self.set_prefs(validator_key, prefs)
    }

    pub fn update_prefs_next(&mut self, validator_key: &AccountKey, prefs: ValidatorPrefs) -> bool {
        self.set_prefs_next(validator_key, prefs)
    }

    pub fn update_points(&mut self, validator_key: &AccountKey, points: Points) -> bool {
        self.set_points(validator_key, points)
    }

    pub fn update_era_points(&mut self, validator_key: &AccountKey, points: Points) -> bool {
        self.set_era_points(validator_key, points)
    }

    pub fn update_identity(&mut self, validator_key: &AccountKey, identity: Identity) {
        self.set_identity(validator_key, identity);
    }

    pub fn update_stake_overview(&mut self, validator_key: &AccountKey, data: StakeOverview) {
        self.set_stake_overview(validator_key, data);
    }

    pub fn update_stake_ledger(&mut self, validator_key: &AccountKey, data: StakeLedger) {
        self.set_stake_ledger(validator_key, data);
    }

    pub fn update_payee(&mut self, validator_key: &AccountKey, data: Payee) {
        self.set_payee(validator_key, data);
    }

    pub fn update_next_keys(&mut self, validator_key: &AccountKey, data: Option<Keys>) {
        self.set_next_keys(validator_key, data);
    }

    pub fn update_queued_keys(&mut self, validator_key: &AccountKey, data: Option<Keys>) {
        self.set_queued_keys(validator_key, data);
    }

    pub fn update_status(&mut self, validator_key: &AccountKey, status: ValidatorStatus) {
        self.set_status(validator_key, status);
    }

    pub fn update_balance(&mut self, validator_key: &AccountKey, balance: Balance) {
        self.set_balance(validator_key, balance);
    }
}
