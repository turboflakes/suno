use crate::widgets::validators_detailed_group::{GROUP_HEADER_HEIGHT, PADDING};
use ratatui::widgets::TableState;
use std::collections::{BTreeMap, HashMap};
use std::time::{SystemTime, UNIX_EPOCH};
use suno_config::{fetch_validators_from_source, NodeConfig, SupportedRuntime, CONFIG};
use suno_primitives::{
    balance::Balance,
    identity::Identity,
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

#[derive(Debug)]
pub struct ValidatorsList {
    pub validators: HashMap<ValidatorKey, Validator>,
    pub validators_order: Vec<ValidatorKey>,
    pub table_state: TableState,
    pub scroll_offset: u16,
    pub viewport_height: u16,
    active: bool,
    masked: bool,
}

impl Default for ValidatorsList {
    fn default() -> Self {
        Self {
            validators: HashMap::new(),
            validators_order: Vec::new(),
            table_state: TableState::default(),
            scroll_offset: 0,
            viewport_height: 0,
            active: false,
            masked: true,
        }
    }
}

impl ValidatorsList {
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
        self.active
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

    pub async fn on_init(&mut self) {
        let chains = CONFIG.chains();
        for chain in chains.iter() {
            for (chain_name, chain_config) in chain {
                // Process validators from the chain config.
                self.process_validators(chain_name, &chain_config.validators);
                // Fetch validators from the external source, if defined.
                if let Some(source) = &chain_config.validators_source {
                    let res = fetch_validators_from_source(source).await;
                    match res {
                        Ok(validators) => {
                            self.process_validators(chain_name, &validators);
                        }
                        Err(e) => error!("{:?}", e),
                    }
                }
            }
        }
        self.init_table();
    }

    fn process_validators(&mut self, chain_name: &SupportedRuntime, validators: &[NodeConfig]) {
        for validator in validators {
            match validator {
                NodeConfig::Address(stash) => {
                    let validator = Validator::new(*chain_name, *stash);
                    self.add_validator(validator);
                }
                NodeConfig::Detailed {
                    stash,
                    host_rpc,
                    ssh,
                    commands,
                    ..
                } => {
                    let mut validator = Validator::new(*chain_name, *stash);
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
        if let Some(selected) = self.table_state.selected() {
            if selected == self.validators_order.len() - 1 {
                self.table_state.select_first();
                self.scroll_offset = 0;
            } else {
                self.table_state.scroll_down_by(1);
            }
            self.ensure_selection_in_view();
            self.table_state
                .selected()
                .and_then(|i| self.get_validator_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn move_up(&mut self) -> Option<Validator> {
        if let Some(selected) = self.table_state.selected() {
            if selected == 0 {
                let i = self.validators_order.len() - 1;
                self.table_state.select(Some(i));
            } else {
                self.table_state.scroll_up_by(1);
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
            self.ensure_selection_in_view();
            self.table_state
                .selected()
                .and_then(|i| self.get_validator_by_index_cloned(i))
        } else {
            None
        }
    }

    pub fn init_table(&mut self) {
        if !self.validators.is_empty() {
            self.table_state.select(Some(0));
        }
    }

    pub fn set_active(&mut self, active: bool) {
        self.active = active;
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

    pub fn is_commands_available(&self) -> bool {
        if let Some(v) = self.get_selected() {
            return v.is_commands_available();
        }
        false
    }

    pub fn get_validator_keys_by_runtime(&self, runtime: SupportedRuntime) -> Vec<AccountKey> {
        self.get_keys_by_runtime(runtime.relay_chain())
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
