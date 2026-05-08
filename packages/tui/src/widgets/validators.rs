use crate::widgets::chains::ChainsListWidget;
use crate::widgets::validators_compact::ValidatorsCompactWidget;
use crate::widgets::validators_detailed_group::{
    ValidatorsDetailedGroupWidget, GROUP_HEADER_HEIGHT, PADDING,
};
use crate::widgets::validators_detailed_list::ValidatorsDetailedListWidget;
use ratatui::widgets::TableState;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
use suno_primitives::{
    balance::Balance,
    identity::Identity,
    proxy::ProxyKey,
    session::Keys,
    staking::{Chunk, Payee, StakeLedger, StakeOverview, ValidatorPrefs},
    validator::{Validator, ValidatorStatus},
    AccountKey,
};

type Points = u32;
type Amount = u128;
type ValidatorKey = AccountKey;

#[derive(Debug, Default)]
pub struct ValidatorsListState {
    pub validators: HashMap<ValidatorKey, Validator>,
    pub validators_order: Vec<ValidatorKey>,
    pub table_state: TableState,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    pub is_active: bool,
}

impl ValidatorsListState {
    pub fn add_validator(&mut self, validator: Validator) {
        let key = validator.key();
        if !self.validators.contains_key(key) {
            self.validators_order.push(key.clone());
        }
        self.validators.insert(key.clone(), validator);
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

    pub fn set_proxies(&mut self, validator_key: &AccountKey, proxies: Vec<ProxyKey>) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.proxies = proxies;
        }
    }

    pub fn add_proxy(&mut self, validator_key: &AccountKey, proxy: ProxyKey) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.proxies.push(proxy);
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

    pub fn is_active(&self) -> bool {
        self.is_active
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
                grouped.entry(key.runtime).or_default().push(validator);
            }
        }

        grouped
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
        let selected_y_position = self.get_selected_y_position();

        if selected_y_position < self.scroll_offset {
            self.scroll_offset = selected_y_position;
        } else if selected_y_position >= self.scroll_offset + self.viewport_height {
            self.scroll_offset = selected_y_position - self.viewport_height + 1;
        }
    }

    // Determine the Y position of the current validator selection
    fn get_selected_y_position(&self) -> u16 {
        let mut selected_y_position = 0;
        let selected_ref = self.get_selected_ref();

        for (_, validators) in self.get_validators_grouped_by_runtime() {
            if let Some(idx) = validators.iter().position(|v| Some(*v) == selected_ref) {
                // Header + index + table header
                return selected_y_position + GROUP_HEADER_HEIGHT + idx as u16 + 1;
            }
            selected_y_position += GROUP_HEADER_HEIGHT + validators.len() as u16 + PADDING;
        }
        0
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
}

#[derive(Debug)]
pub struct ValidatorsListWidget {
    state: Arc<RwLock<ValidatorsListState>>,
    // The sender to send actions to update the state to the app.
    // tx: UnboundedSender<Action>,
}

impl<'a> ValidatorsListWidget {
    // Add methods to create the alternative widgets
    pub fn as_compact(&self) -> ValidatorsCompactWidget {
        ValidatorsCompactWidget {
            state: self.state.clone(),
        }
    }

    pub fn as_detailed_group(
        &self,
        chains: &'a ChainsListWidget,
    ) -> ValidatorsDetailedGroupWidget<'a> {
        ValidatorsDetailedGroupWidget {
            state: self.state.clone(),
            chains,
        }
    }

    pub fn as_detailed_list(&self) -> ValidatorsDetailedListWidget {
        ValidatorsDetailedListWidget {
            state: self.state.clone(),
        }
    }
}

impl Default for ValidatorsListWidget {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorsListWidget {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(ValidatorsListState::default())),
            // tx,
        }
    }

    pub fn on_init(&self) {
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                for validator in &chain_config.validators {
                    match validator {
                        NodeConfig::Address(stash) => {
                            let validator = Validator::new(*chain_name, *stash);
                            self.add_validator(&validator);
                        }
                        NodeConfig::Detailed {
                            stash,
                            host,
                            commands,
                        } => {
                            let mut validator = Validator::new(*chain_name, *stash);
                            validator.host = host.clone();
                            if let Some(cmds) = commands {
                                validator.commands = cmds.clone();
                            }
                            self.add_validator(&validator);
                        }
                    }
                }
            }
        }
        self.init_table();
    }

    fn add_validator(&self, validator: &Validator) {
        let mut state = self.state.write().unwrap();
        state.add_validator(validator.clone());
    }

    pub fn move_down(&self) -> Option<Validator> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == state.validators_order.len() - 1 {
                state.table_state.select_first();
                state.scroll_offset = 0;
            } else {
                state.table_state.scroll_down_by(1);
            }
            state.ensure_selection_in_view();
            state
                .table_state
                .selected()
                .and_then(|i| state.get_validator_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn move_up(&self) -> Option<Validator> {
        let mut state = self.state.write().unwrap();
        if let Some(selected) = state.table_state.selected() {
            if selected == 0 {
                let i = state.validators_order.len() - 1;
                state.table_state.select(Some(i));
            } else {
                state.table_state.scroll_up_by(1);
                state.scroll_offset = state.scroll_offset.saturating_sub(1);
            }
            state.ensure_selection_in_view();
            state
                .table_state
                .selected()
                .and_then(|i| state.get_validator_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn init_table(&self) {
        let mut state = self.state.write().unwrap();
        if !state.validators.is_empty() {
            state.table_state.select(Some(0));
        }
    }

    pub fn is_active(&self) -> bool {
        let state = self.state.read().unwrap();
        state.is_active()
    }

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Validator> {
        let state = self.state.read().unwrap();
        state.get_selected()
    }

    pub fn is_proxy_valid(&self) -> bool {
        let state = self.state.read().unwrap();
        if let Some(v) = state.get_selected() {
            return v.is_proxy_valid();
        }
        false
    }

    pub fn is_commands_available(&self) -> bool {
        let state = self.state.read().unwrap();
        if let Some(v) = state.get_selected() {
            return v.is_commands_available();
        }
        false
    }
    
    

    pub fn get_validator_keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        let state = self.state.read().unwrap();
        state.get_keys_by_runtime(runtime.relay_chain())
    }

    pub fn update_prefs(&self, validator_key: &AccountKey, prefs: ValidatorPrefs) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_prefs(validator_key, prefs)
    }

    pub fn update_prefs_next(&self, validator_key: &AccountKey, prefs: ValidatorPrefs) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_prefs_next(validator_key, prefs)
    }

    pub fn update_points(&self, validator_key: &AccountKey, points: Points) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_points(validator_key, points)
    }

    pub fn update_era_points(&self, validator_key: &AccountKey, points: Points) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_era_points(validator_key, points)
    }

    pub fn update_identity(&self, validator_key: &AccountKey, identity: Identity) {
        let mut state = self.state.write().unwrap();
        state.set_identity(validator_key, identity);
    }

    pub fn update_stake_overview(&self, validator_key: &AccountKey, data: StakeOverview) {
        let mut state = self.state.write().unwrap();
        state.set_stake_overview(validator_key, data);
    }

    pub fn update_stake_ledger(&self, validator_key: &AccountKey, data: StakeLedger) {
        let mut state = self.state.write().unwrap();
        state.set_stake_ledger(validator_key, data);
    }

    pub fn update_payee(&self, validator_key: &AccountKey, data: Payee) {
        let mut state = self.state.write().unwrap();
        state.set_payee(validator_key, data);
    }

    pub fn update_next_keys(&self, validator_key: &AccountKey, data: Option<Keys>) {
        let mut state = self.state.write().unwrap();
        state.set_next_keys(validator_key, data);
    }

    pub fn update_queued_keys(&self, validator_key: &AccountKey, data: Option<Keys>) {
        let mut state = self.state.write().unwrap();
        state.set_queued_keys(validator_key, data);
    }

    pub fn update_status(&self, validator_key: &AccountKey, status: ValidatorStatus) {
        let mut state = self.state.write().unwrap();
        state.set_status(validator_key, status);
    }

    pub fn add_amount_to_stake_ledger(&self, validator_key: &AccountKey, amount: Amount) {
        let mut state = self.state.write().unwrap();
        state.add_amount_to_stake_ledger(validator_key, amount);
    }

    pub fn sub_chunk_from_stake_ledger(&self, validator_key: &AccountKey, chunk: Chunk) {
        let mut state = self.state.write().unwrap();
        state.sub_chunk_from_stake_ledger(validator_key, chunk);
    }

    pub fn add_proxy(&self, validator_key: &AccountKey, proxy: ProxyKey) {
        let mut state = self.state.write().unwrap();
        state.add_proxy(validator_key, proxy);
    }

    pub fn update_balance(&self, validator_key: &AccountKey, balance: Balance) {
        let mut state = self.state.write().unwrap();
        state.set_balance(validator_key, balance);
    }

    pub fn add_amount_to_balance(&self, validator_key: &AccountKey, amount: Amount) {
        let mut state = self.state.write().unwrap();
        state.add_amount_to_balance(validator_key, amount);
    }
}
