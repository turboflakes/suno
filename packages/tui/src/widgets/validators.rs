use crate::error::TuiError;
use crate::widgets::chains::ChainsListWidget;
use crate::widgets::validators_compact::ValidatorsCompactWidget;
use crate::widgets::validators_detailed_group::{
    ValidatorsDetailedGroupWidget, BOTTOM_PADDING, GROUP_HEADER_HEIGHT,
};
use crate::widgets::validators_detailed_list::ValidatorsDetailedListWidget;
// use crate::widgets::popup::PopupWidget;
use crate::bridge::dispatch::dispatch_response_action;
use futures::{
    future::{BoxFuture, FutureExt},
    select, stream,
    stream::{FuturesUnordered, StreamExt},
};
use log::{error, info, warn};
use ratatui::widgets::TableState;
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};
use subxt::utils::H256;
use subxt::{OnlineClient, SubstrateConfig};
use suno_actions::{Action, ChainAction, SystemAction, ValidatorAction};
use suno_asset_hub_paseo;
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
use suno_error::Error;
use suno_primitives::{
    staking::{Era, StakeLedger, StakeOverview},
    validator::{Validator, ValidatorStatus},
    AccountKey, Response,
};
use tokio::sync::mpsc::UnboundedSender;

type Commission = u32;
type Points = u32;
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
        if !self.validators.contains_key(&key) {
            self.validators_order.push(key.clone());
        }
        self.validators.insert(key.clone(), validator);
    }

    pub fn set_commission(&mut self, validator_key: &AccountKey, commission: Commission) -> bool {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            if validator.commission != commission {
                validator.commission = commission;
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

    pub fn set_identity(&mut self, validator_key: &AccountKey, identity: String) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.account.set_identity(identity);
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
        }
    }

    pub fn set_status(&mut self, validator_key: &AccountKey, status: ValidatorStatus) {
        if let Some(validator) = self.validators.get_mut(validator_key) {
            validator.status = status;
        }
    }

    pub fn set_viewport_height(&mut self, height: u16) {
        self.viewport_height = height;
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
    pub fn get_keys_by_runtime(&self, runtime: &SupportedRuntime) -> Vec<AccountKey> {
        self.validators_order
            .iter()
            .filter(|key| &key.runtime == runtime)
            .cloned()
            .collect()
    }

    pub fn get_keys_grouped_by_runtime_cloned(&self) -> HashMap<SupportedRuntime, Vec<AccountKey>> {
        let mut grouped: HashMap<SupportedRuntime, Vec<AccountKey>> = HashMap::new();

        for (key, _) in &self.validators {
            grouped
                .entry(key.runtime.clone())
                .or_insert_with(Vec::new)
                .push(key.clone());
        }

        grouped
    }

    pub fn get_validators_grouped_by_runtime(&self) -> BTreeMap<SupportedRuntime, Vec<&Validator>> {
        let mut grouped: BTreeMap<SupportedRuntime, Vec<&Validator>> = BTreeMap::new();

        for key in &self.validators_order {
            if let Some(validator) = self.get_validator_by_key(key) {
                grouped
                    .entry(key.runtime.clone())
                    .or_insert_with(Vec::new)
                    .push(validator);
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
            .iter()
            .map(|(_, v)| GROUP_HEADER_HEIGHT + v.len() as u16 + BOTTOM_PADDING)
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
            selected_y_position += GROUP_HEADER_HEIGHT + validators.len() as u16 + BOTTOM_PADDING;
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
    /// The sender to send actions to update the state to the app.
    tx: UnboundedSender<Action>,
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

impl ValidatorsListWidget {
    pub fn new(tx: UnboundedSender<Action>) -> Self {
        Self {
            state: Arc::new(RwLock::new(ValidatorsListState::default())),
            tx,
        }
    }

    pub fn on_init(&self) {
        let config = CONFIG.clone();
        for chain in config.chains.iter() {
            for (chain_name, chain_config) in chain {
                for validator in &chain_config.validators {
                    match validator {
                        NodeConfig::Address(stash) => {
                            let validator = Validator::new(chain_name.clone(), stash.clone());
                            self.add_validator(&validator);
                        }
                        NodeConfig::Detailed { stash, .. } => {
                            let validator = Validator::new(chain_name.clone(), stash.clone());
                            self.add_validator(&validator);

                            // TODO: Implement command handling
                            // if let Some(cmds) = commands {
                            //     for cmd in cmds {
                            //         println!("  Command: {} ({})", cmd.name, cmd.run);
                            //     }
                            // }
                        }
                    }
                }
            }
        }
        self.init_table();
    }

    fn on_error(&self, err: Box<dyn std::error::Error>) {
        self.tx
            .send(Action::System(SystemAction::Error(err.to_string())))
            .expect("Failed to send error message");
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

    pub fn set_active(&self, active: bool) {
        let mut state = self.state.write().unwrap();
        state.is_active = active;
    }

    pub fn get_selected(&self) -> Option<Validator> {
        let state = self.state.read().unwrap();
        state.get_selected()
    }

    pub fn get_validator_keys_by_runtime(&self, runtime: &SupportedRuntime) -> Vec<AccountKey> {
        let state = self.state.read().unwrap();
        state.get_keys_by_runtime(&runtime.relay_chain())
    }

    pub fn update_commission(&self, validator_key: &AccountKey, commission: Commission) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_commission(validator_key, commission)
    }

    pub fn update_points(&self, validator_key: &AccountKey, points: Points) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_points(validator_key, points)
    }

    pub fn update_era_points(&self, validator_key: &AccountKey, points: Points) -> bool {
        let mut state = self.state.write().unwrap();
        state.set_era_points(validator_key, points)
    }

    pub fn update_identity(&self, validator_key: &AccountKey, identity: String) {
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

    pub fn update_status(&self, validator_key: &AccountKey, status: ValidatorStatus) {
        let mut state = self.state.write().unwrap();
        state.set_status(validator_key, status);
    }

    // // DEPRECATED
    // fn fetch_validator_data(&self, validator: &Validator) {
    //     self.tx
    //         .send(Action::Chain(ChainAction::FetchValidatorData(
    //             validator.key().clone(),
    //         )))
    //         .unwrap_or_else(|err| self.on_error(err.into()));
    // }

    // // DEPRECATED
    // fn fetch_all_validators_data(&self) {
    //     let state = self.state.read().unwrap();
    //     let keys_grouped = state.get_keys_grouped_by_runtime_cloned();
    //     keys_grouped.into_iter().for_each(|(runtime, keys)| {
    //         self.tx
    //             .send(Action::Chain(ChainAction::FetchValidatorsData(
    //                 runtime, keys,
    //             )))
    //             .unwrap_or_else(|err| self.on_error(err.into()));
    //     });
    // }

    // TODO
    // pub fn chill(&self, chain: &Chain, tx: UnboundedSender<Action>) {
    //     if chain.is_offline() {
    //         warn!("TODO: Chain {} not ready", chain.runtime());
    //         return;
    //     }

    //     let api = chain.client().clone();
    //     let runtime = self.runtime().clone();
    //     let tx = tx.clone();
    //     let stash = self.account.stash().clone();
    //     tokio::spawn(async move {
    //         // let response = match runtime {
    //         //     SupportedRuntime::Westend => {
    //         //         // TODO: Implement password input for proxy signing
    //         //         let chill_xt = suno_westend::staking::chill();
    //         //         suno_westend::submit_as_proxy(&api, chill_xt, stash, None, tx).await
    //         //     }
    //         //     _ => unimplemented!("Chill not implemented for {:?}", runtime),
    //         // };
    //         // match response {
    //         //     Err(e) => {
    //         //         warn!("TODO: error: {:?}", e);
    //         //     }
    //         //     _ => (),
    //         // }
    //     });
    // }

    pub fn spawn_fetch_validators_commission(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        runtime: &SupportedRuntime,
    ) {
        let state = self.state.read().unwrap();
        let validator_keys = state.get_keys_by_runtime(&runtime.relay_chain());
        let api = api.clone();
        let runtime = runtime.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(e) = fetch_and_send_validators_commission(
                &api,
                block_hash,
                &runtime,
                validator_keys.clone(),
                tx.clone(),
            )
            .await
            {
                let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
            }
        });
    }

    pub fn spawn_fetch_validators_identities(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        runtime: &SupportedRuntime,
    ) {
        let state = self.state.read().unwrap();
        let validator_keys = state.get_keys_by_runtime(&runtime.relay_chain());
        let api = api.clone();
        let runtime = runtime.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(e) = fetch_and_send_validators_identities(
                &api,
                block_hash,
                &runtime,
                validator_keys,
                tx.clone(),
            )
            .await
            {
                let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
            }
        });
    }

    pub fn spawn_fetch_validators_authority_status_from_relay(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        runtime: &SupportedRuntime,
    ) {
        let state = self.state.read().unwrap();
        let validator_keys = state.get_keys_by_runtime(&runtime.relay_chain());
        let api = api.clone();
        let runtime = runtime.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(e) = fetch_and_dispatch_validators_authority_status(
                &api,
                block_hash,
                &runtime,
                &validator_keys,
                &tx,
            )
            .await
            {
                let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
            }
        });
    }
}

// Helper functions

// Helper functions to fetch all types of validator data in parallel and without overflowing the RPCs
// Useful when a large list of validators is configured
const CONCURRENT_REQUESTS: usize = 3;

async fn fetch_and_send_validators_identities(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: Vec<AccountKey>,
    tx: UnboundedSender<Action>,
) -> Result<(), TuiError> {
    let mut stream = stream::iter(validator_keys)
        .map(|validator_key| {
            let api = api.clone();
            let stash = validator_key.stash();
            let runtime = runtime.clone();
            async move {
                let result = match runtime {
                    SupportedRuntime::PeoplePolkadot => {
                        suno_people_polkadot::fetch_display_name(&api, block_hash, &stash).await
                    }
                    SupportedRuntime::PeopleKusama => {
                        suno_people_kusama::fetch_display_name(&api, block_hash, &stash).await
                    }
                    SupportedRuntime::PeoplePaseo => {
                        suno_people_paseo::fetch_display_name(&api, block_hash, &stash).await
                    }
                    SupportedRuntime::PeopleWestend => {
                        suno_people_westend::fetch_display_name(&api, block_hash, &stash).await
                    }
                    _ => Err(suno_error::Error::from("Unsupported runtime")),
                };
                (validator_key, result)
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some((validator_key, result)) = stream.next().await {
        match result {
            Ok(identity) => {
                tx.send(Action::Validator(ValidatorAction::UpdateIdentity(
                    validator_key.clone(),
                    identity,
                )))?;
            }
            Err(e) => warn!("{e}"),
        }
    }

    Ok(())
}

async fn fetch_and_send_validators_commission(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: Vec<ValidatorKey>,
    tx: UnboundedSender<Action>,
) -> Result<(), TuiError> {
    let mut stream = stream::iter(validator_keys)
        .map(|validator_key| {
            let api = api.clone();
            let stash = validator_key.stash();
            let runtime = runtime.clone();
            async move {
                let result = match runtime {
                    SupportedRuntime::AssetHubPolkadot => {
                        suno_asset_hub_polkadot::fetch_validator_commission(
                            &api, block_hash, &stash,
                        )
                        .await
                    }
                    SupportedRuntime::AssetHubKusama => {
                        suno_asset_hub_kusama::fetch_validator_commission(&api, block_hash, &stash)
                            .await
                    }
                    SupportedRuntime::AssetHubPaseo => {
                        suno_asset_hub_paseo::fetch_validator_commission(&api, block_hash, &stash)
                            .await
                    }
                    SupportedRuntime::AssetHubWestend => {
                        suno_asset_hub_westend::fetch_validator_commission(&api, block_hash, &stash)
                            .await
                    }
                    _ => Err(suno_error::Error::from("Unsupported runtime")),
                };
                (validator_key, result)
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some((validator_key, result)) = stream.next().await {
        match result {
            Ok(commission) => {
                tx.send(Action::Validator(ValidatorAction::UpdateCommission(
                    validator_key.clone(),
                    commission,
                )))?;
            }
            Err(e) => {
                // Note: When an error is reported here it could be a connectivity issue,
                // or that the commission was never set for the specific stash, so we set the
                // validator status to unknown.
                tx.send(Action::Validator(ValidatorAction::UpdateStatus(
                    validator_key.clone(),
                    ValidatorStatus::Unknown,
                )))?;
                warn!("{e}")
            }
        }
    }

    Ok(())
}

async fn fetch_and_dispatch_validators_authority_status(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: &Vec<ValidatorKey>,
    tx: &UnboundedSender<Action>,
) -> Result<(), Error> {
    let responses = match runtime {
        SupportedRuntime::Polkadot => {
            suno_polkadot::fetch_validators_authority_status(api, block_hash, validator_keys)
                .await?
        }
        SupportedRuntime::Kusama => {
            suno_kusama::fetch_validators_authority_status(api, block_hash, validator_keys).await?
        }
        SupportedRuntime::Paseo => {
            suno_paseo::fetch_validators_authority_status(api, block_hash, validator_keys).await?
        }
        SupportedRuntime::Westend => {
            suno_westend::fetch_validators_authority_status(api, block_hash, validator_keys).await?
        }
        _ => {
            return Err(Error::UnsupportedRuntime(runtime.clone()));
        }
    };

    for response in responses {
        dispatch_response_action(response, runtime, tx)?;
    }

    Ok(())
}
