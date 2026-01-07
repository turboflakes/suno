use crate::error::TuiError;
use crate::theme::THEME;
use crate::widgets::chains::{Chain, ChainsListWidget};
use crate::widgets::popup::PopupWidget;
use crate::widgets::scrollbar::render_scrollbar;
use futures::{
    future::{BoxFuture, FutureExt},
    select, stream, Future, StreamExt,
};
use log::{error, info, warn};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    prelude::Stylize,
    style::{Color, Modifier, Style, Styled},
    text::{Line, Text},
    widgets::{
        Block, BorderType, Borders, Cell, Paragraph, Row, StatefulWidget, Table, TableState, Widget,
    },
};
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use subxt::utils::{AccountId32, H256};
use subxt::{OnlineClient, SubstrateConfig};
use suno_actions::{Action, ChainAction, SystemAction, ValidatorAction};
use suno_asset_hub_paseo;
use suno_config::{NodeConfig, SupportedRuntime, CONFIG};
use suno_error::Error;
use suno_primitives::{
    display::{create_progress_bar_by_blocks, format_planks, get_elapsed_millis},
    staking::{Era, StakeLedger, StakeOverview},
    AccountDisplay, AccountKey, NodeAccount,
};
use tokio::sync::mpsc::UnboundedSender;

type Commission = u32;
type Points = u32;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ValidatorState {
    /// Validator is an authority in the active set
    Authority,
    /// Validator is an authority and also a parachain authority
    ParaAuthority,
    /// Validator is in the waiting queue
    Waiting,
    /// Validator status is unknown or not yet determined
    #[default]
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Nominators {
    pub stash: AccountId32,
    pub stake: u128,
    pub is_backer: bool,
}

#[derive(Debug, Clone)]
pub struct Validator {
    pub account: NodeAccount,
    pub commission: Commission,
    pub stake: StakeOverview,
    pub ledger: StakeLedger,
    pub nominators: Vec<Nominators>,
    // Track session points from staking_ah_client.validator_points
    pub points: Points,
    // Track old points so it can be better rendered the delta points
    pub old_points: Points,
    pub old_points_ts: u128,
    // Track era points accumulated at every new session from staking.era_reward_points
    // the total points earned at any single time will be sum of points + era_points
    pub era_points: Points,
    pub is_next_authority: bool,
    pub is_chilled: bool,
    pub state: ValidatorState,
}

impl Validator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
            commission: 0,
            stake: StakeOverview::default(),
            ledger: StakeLedger::default(),
            nominators: Vec::new(),
            points: 0,
            old_points: 0,
            old_points_ts: 0,
            era_points: 0,
            is_next_authority: false,
            is_chilled: false,
            state: ValidatorState::default(),
        }
    }

    pub fn key(&self) -> &AccountKey {
        &self.account.account_key()
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.account.runtime()
    }

    pub fn identity(&self) -> &Option<String> {
        self.account.identity()
    }

    pub fn display_name(&self) -> String {
        if let Some(display_name) = self.identity() {
            display_name.clone()
        } else {
            self.to_compact_string(6)
        }
    }

    pub fn commission_as_percentage(&self, decimal_places: usize) -> String {
        let percentage = self.commission as f64 / 10_000_000.0;
        let formatted = format!("{:.prec$}", percentage, prec = decimal_places);
        let trimmed = formatted.trim_end_matches('0').trim_end_matches('.');
        format!("{}%", trimmed)
    }

    pub fn points(&self) -> Points {
        self.points
    }

    pub fn total_points(&self) -> Points {
        self.points + self.era_points
    }

    pub fn delta_points(&self) -> Option<Points> {
        if self.points <= self.old_points {
            return None;
        }
        let elapsed = get_elapsed_millis(self.old_points_ts);
        if elapsed >= 2_000 {
            return None;
        }
        return Some(self.points - self.old_points);
    }

    pub fn chill(&self, chain: &Chain, tx: UnboundedSender<Action>) {
        if chain.is_offline() {
            warn!("TODO: Chain {} not ready", chain.runtime());
            return;
        }

        let api = chain.client().clone();
        let runtime = self.runtime().clone();
        let tx = tx.clone();
        let stash = self.account.stash().clone();
        tokio::spawn(async move {
            // let response = match runtime {
            //     SupportedRuntime::Westend => {
            //         // TODO: Implement password input for proxy signing
            //         let chill_xt = suno_westend::staking::chill();
            //         suno_westend::submit_as_proxy(&api, chill_xt, stash, None, tx).await
            //     }
            //     _ => unimplemented!("Chill not implemented for {:?}", runtime),
            // };
            // match response {
            //     Err(e) => {
            //         warn!("TODO: error: {:?}", e);
            //     }
            //     _ => (),
            // }
        });
    }
}

impl AccountDisplay for Validator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }
}

type ValidatorKey = AccountKey;

#[derive(Debug, Default)]
pub struct ValidatorsListState {
    validators: HashMap<ValidatorKey, Validator>,
    validators_order: Vec<ValidatorKey>,
    table_state: TableState,
    table_unselected_rows_indices: Vec<usize>,
    is_active: bool,
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
}

#[derive(Debug)]
pub struct ValidatorsListWidget {
    state: Arc<RwLock<ValidatorsListState>>,
    /// The sender to send actions to update the state to the app.
    tx: UnboundedSender<Action>,
}

#[derive(Debug, Clone)]
pub struct ValidatorsCompactWidget {
    state: Arc<RwLock<ValidatorsListState>>,
}

#[derive(Debug, Clone)]
pub struct ValidatorsDetailWidget<'a> {
    state: Arc<RwLock<ValidatorsListState>>,
    chains: &'a ChainsListWidget,
}

impl<'a> ValidatorsListWidget {
    // Add methods to create the alternative widgets
    pub fn as_compact(&self) -> ValidatorsCompactWidget {
        ValidatorsCompactWidget {
            state: self.state.clone(),
        }
    }

    pub fn as_detail(&self, chains: &'a ChainsListWidget) -> ValidatorsDetailWidget<'a> {
        ValidatorsDetailWidget {
            state: self.state.clone(),
            chains,
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
            } else {
                state.table_state.scroll_down_by(1);
            }
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
            }
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
        state
            .table_state
            .selected()
            .and_then(|i| state.get_validator_by_index_cloned(i))
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

    // DEPRECATED
    fn fetch_validator_data(&self, validator: &Validator) {
        self.tx
            .send(Action::Chain(ChainAction::FetchValidatorData(
                validator.key().clone(),
            )))
            .unwrap_or_else(|err| self.on_error(err.into()));
    }

    // DEPRECATED
    fn fetch_all_validators_data(&self) {
        let state = self.state.read().unwrap();
        let keys_grouped = state.get_keys_grouped_by_runtime_cloned();
        keys_grouped.into_iter().for_each(|(runtime, keys)| {
            self.tx
                .send(Action::Chain(ChainAction::FetchValidatorsData(
                    runtime, keys,
                )))
                .unwrap_or_else(|err| self.on_error(err.into()));
        });
    }

    pub fn spawn_fetch_initial_data_from_asset_hub(
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
            if let Err(e) = fetch_and_send_initial_data_from_asset_hub(
                &api,
                block_hash,
                &runtime,
                &validator_keys,
                tx.clone(),
            )
            .await
            {
                let _ = tx.send(Action::System(SystemAction::Error(e.to_string())));
            }
        });
    }

    pub fn spawn_fetch_validators_staking_ledger(
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
            if let Err(e) = fetch_and_send_validators_staking_ledger(
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

    pub fn spawn_fetch_validators_stake_overview(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        runtime: &SupportedRuntime,
        era_index: u32,
    ) {
        let state = self.state.read().unwrap();
        let validator_keys = state.get_keys_by_runtime(&runtime.relay_chain());
        let api = api.clone();
        let runtime = runtime.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(e) = fetch_and_send_validators_stake_overview(
                &api,
                block_hash,
                &runtime,
                era_index,
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

    pub fn spawn_fetch_validators_points_from_relay(
        &self,
        api: &OnlineClient<SubstrateConfig>,
        block_hash: H256,
        runtime: &SupportedRuntime,
    ) {
        let state = self.state.read().unwrap();
        let validator_keys = state.get_keys_by_runtime(runtime);
        let api = api.clone();
        let runtime = runtime.clone();
        let tx = self.tx.clone();

        tokio::spawn(async move {
            if let Err(e) = fetch_and_send_validators_points_from_relay(
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
}

// Compact widget implementation, mostly to be used on the left menu
impl Widget for &ValidatorsCompactWidget {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let mut state = self.state.write().unwrap();

        let (table_style, highlight_style, highlight_symbol) = match state.is_active {
            true => (
                Style::default().fg(Color::White),
                Style::default().fg(Color::Black).bg(Color::White),
                "❯ ",
            ),
            false => (
                Style::default().fg(Color::Blue),
                Style::default().fg(Color::Blue),
                "",
            ),
        };

        let block = Block::new()
            .title("Validators")
            .title_style(Style::default().add_modifier(Modifier::BOLD))
            .borders(Borders::LEFT | Borders::BOTTOM)
            .border_type(BorderType::Plain);

        let rows = state.validators_iter();

        let widths = [
            Constraint::Fill(1),    // Network column
            Constraint::Length(14), // Stash column
        ];

        let table = Table::new(rows, widths)
            .block(block)
            .style(table_style)
            .row_highlight_style(highlight_style)
            .highlight_symbol(highlight_symbol);

        StatefulWidget::render(table, area, buf, &mut state.table_state);

        // // Render scrollbar when active
        // if state.is_active {

        //     let scrollbar_area = Rect {
        //         x: area.x,
        //         y: area.y + 1,
        //         width: 1,
        //         height: area.height - 2,
        //         ..area
        //     };
        //     if let Some(row_index) = state.table_state.selected() {
        //         render_scrollbar(row_index, state.validators.len(), scrollbar_area, buf);
        //     }
        // }
    }
}

// Detailed widget implementation, with all relevant information
impl<'a> Widget for &ValidatorsDetailWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let state = self.state.write().unwrap();

        // Split area into sections for each runtime group
        let grouped = state.get_validators_grouped_by_runtime();

        // Calculate heights for each section
        let mut constraints = Vec::new();
        for (_, validators) in &grouped {
            let group_height = 6 + validators.len() as u16;
            constraints.push(Constraint::Length(group_height));
        }

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        for (i, (runtime, validators)) in grouped.into_iter().enumerate() {
            let group_area = chunks[i];

            // Split group area into header and body
            let group_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(4), // Header height
                    Constraint::Min(0),    // Body takes remaining
                ])
                .split(group_area);

            // Render header with custom layout
            self.render_table_header(runtime, group_chunks[0], buf);

            // Render validators table
            self.render_table_body(
                validators,
                group_chunks[1],
                buf,
                &mut state.table_state.clone(),
            );
        }
    }
}

impl<'a> ValidatorsDetailWidget<'a> {
    fn render_table_header(&self, runtime: SupportedRuntime, area: Rect, buf: &mut Buffer) {
        if let Some(chain) = self.chains.get_chain_by_runtime(&runtime) {
            if let Some(ah_chain) = self
                .chains
                .get_chain_by_runtime(&runtime.asset_hub_runtime())
            {
                let header_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([
                        Constraint::Length(28), // Network info
                        Constraint::Fill(1),    // Era / Session progress bar
                        Constraint::Length(16), // Countdown
                    ])
                    .split(area);

                let network_info = Paragraph::new(vec![
                    Line::from(format!("# {}", runtime))
                        .style(Style::default().fg(Color::Blue).bold()),
                    Line::from(format!(
                        "validators: {}/{}",
                        ah_chain.active_validators(),
                        ah_chain.total_validators()
                    )),
                    Line::from(format!(
                        "nominators: {}/{}",
                        ah_chain.active_nominators(),
                        ah_chain.total_nominators()
                    )),
                    // TODO: Get onchain data
                    Line::from(format!("staked: {:.2}%", 55.0)),
                ])
                .style(Style::default().fg(Color::Blue));

                network_info.render(header_layout[0], buf);

                let Some(epoch) = chain.epoch() else {
                    // TODO: Handle epoch not available, maybe render loading indicator
                    return;
                };

                let epoch_progress = epoch.progress(chain.finalized_block());
                let epoch_progress_bar = create_progress_bar_by_blocks(epoch_progress, 24);

                let Some(era) = ah_chain.era() else {
                    // TODO: Handle era not available, maybe render loading indicator
                    return;
                };

                let era_progress = era.progress(epoch.duration(), epoch.block_time_ms());
                let era_progress_bar = create_progress_bar_by_blocks(era_progress, 24);

                let progress_info = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(format!(
                        "era {} {:.0}% {}",
                        era.index(),
                        era_progress * 100 as f64,
                        era_progress_bar
                    ))
                    .alignment(Alignment::Right),
                    Line::from(format!(
                        "epoch {} {:.0}% {}",
                        epoch.index(),
                        epoch_progress * 100 as f64,
                        epoch_progress_bar,
                    ))
                    .alignment(Alignment::Right),
                ])
                .style(Style::default().fg(Color::Blue));

                progress_info.render(header_layout[1], buf);

                let epoch_countdown_time = epoch.countdown_time(chain.finalized_block());
                let era_countdown_time =
                    era.countdown_time(epoch.duration(), epoch.block_time_ms());

                let countdown_info = Paragraph::new(vec![
                    Line::from(""),
                    Line::from(format!(" {}", era_countdown_time,)).alignment(Alignment::Left),
                    Line::from(format!(" {}", epoch_countdown_time,)).alignment(Alignment::Left),
                ])
                .style(Style::default().fg(Color::Blue));

                countdown_info.render(header_layout[2], buf);
            };
        };
    }

    fn render_table_body(
        &self,
        validators: Vec<&Validator>,
        area: Rect,
        buf: &mut Buffer,
        table_state: &mut TableState,
    ) {
        let mut rows = Vec::new();

        for v in validators {
            let text_points = match v.delta_points() {
                Some(d) => Text::from(format!("+{}", d)).style(Style::default().fg(Color::White)),
                None => Text::from(v.total_points().to_string()),
            };

            let decimals = v.runtime().token_decimals();
            let validator_row = Row::new(vec![
                Text::from(format!("{}", v.display_name())).alignment(Alignment::Left),
                text_points.alignment(Alignment::Right),
                Text::from(format_planks(v.stake.total(), decimals, 4)).alignment(Alignment::Right),
                Text::from(format_planks(v.stake.own(), decimals, 4)).alignment(Alignment::Right),
                Text::from(v.stake.nominators_count().to_string()).alignment(Alignment::Right),
                Text::from(v.commission_as_percentage(2)).alignment(Alignment::Right),
            ]);
            rows.push(validator_row);
        }

        let widths = [
            Constraint::Length(28),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
            Constraint::Fill(1),
        ];

        let (table_style, highlight_style) = (
            Style::default().fg(Color::Blue),
            Style::default().fg(Color::Blue),
        );

        let table = Table::new(rows, widths)
            .header(
                Row::new(vec![
                    Cell::from(""),
                    Cell::from(Text::from("points").alignment(Alignment::Right)),
                    Cell::from(Text::from("total").alignment(Alignment::Right)),
                    Cell::from(Text::from("own").alignment(Alignment::Right)),
                    Cell::from(Text::from("nominators").alignment(Alignment::Right)),
                    Cell::from(Text::from("commission").alignment(Alignment::Right)),
                ])
                .set_style(THEME.table.header),
            )
            .style(table_style)
            .row_highlight_style(highlight_style);

        StatefulWidget::render(table, area, buf, table_state);
    }
}

// impl Widget for &ValidatorsDetailWidget {
//     fn render(self, area: Rect, buf: &mut Buffer) {
//         let mut state = self.state.write().unwrap();

//         let (table_style, highlight_style) = match state.is_active {
//             true => (
//                 Style::default().fg(Color::White),
//                 Style::default().fg(Color::Black).bg(Color::White),
//             ),
//             false => (
//                 Style::default().fg(Color::Blue),
//                 Style::default().fg(Color::Blue),
//             ),
//         };

//         let block = Block::new()
//             .borders(Borders::NONE)
//             .border_type(BorderType::Plain);

//         let rows = state.validators_iter().map(|v| {
//             let points = match v.delta_points() {
//                 Some(d) => format!("+{} {}", d, v.total_points()),
//                 None => v.total_points().to_string(),
//             };

//             let decimals = v.account.token_decimals();
//             Row::new(vec![
//                 Text::from(v.display_name()).alignment(Alignment::Left),
//                 Text::from(points).alignment(Alignment::Right),
//                 Text::from(format_planks(v.stake.total(), decimals, 4)).alignment(Alignment::Right),
//                 Text::from(format_planks(v.stake.own(), decimals, 4)).alignment(Alignment::Right),
//                 Text::from(v.stake.nominators_count().to_string()).alignment(Alignment::Right),
//                 Text::from(v.commission_as_percentage(2)).alignment(Alignment::Right),
//             ])
//         });

//         let widths = [
//             Constraint::Length(20),
//             Constraint::Fill(1),
//             Constraint::Fill(1),
//             Constraint::Fill(1),
//             Constraint::Fill(1),
//             Constraint::Fill(1),
//         ];

//         let table = Table::new(rows, widths)
//             .block(block)
//             .header(
//                 Row::new(vec![
//                     Cell::from(""),
//                     Cell::from(Text::from("points").alignment(Alignment::Right)),
//                     Cell::from(Text::from("total").alignment(Alignment::Right)),
//                     Cell::from(Text::from("own").alignment(Alignment::Right)),
//                     Cell::from(Text::from("nominators").alignment(Alignment::Right)),
//                     Cell::from(Text::from("commission").alignment(Alignment::Right)),
//                 ])
//                 .set_style(THEME.table.header),
//             )
//             .style(table_style)
//             .row_highlight_style(highlight_style);

//         StatefulWidget::render(table, area, buf, &mut state.table_state);
//     }
// }

impl From<&Validator> for Row<'_> {
    fn from(v: &Validator) -> Self {
        // TODO: Verify if proxy is available and correctly setup for each stash
        let has_proxy = false;
        let status = if has_proxy { "[P]" } else { "[R]" };
        let v = v.clone();
        Row::new(vec![
            Text::from(format!("{}/{}", v.runtime(), v.display_name(),)),
            Text::from(status).alignment(Alignment::Right),
        ])
    }
}

// Helper functions

async fn fetch_and_send_initial_data_from_asset_hub(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    validator_keys: &Vec<AccountKey>,
    tx: UnboundedSender<Action>,
) -> Result<(), TuiError> {
    let (
        era_data_fut,
        validators_era_points_fut,
        active_validators_count_fut,
        total_validators_count_fut,
        active_nominators_count_fut,
        total_nominators_count_fut,
    ): (
        BoxFuture<'_, Result<Era, Error>>,
        BoxFuture<'_, Result<HashMap<[u8; 32], u32>, Error>>,
        BoxFuture<'_, Result<u32, Error>>,
        BoxFuture<'_, Result<u32, Error>>,
        BoxFuture<'_, Result<u32, Error>>,
        BoxFuture<'_, Result<u32, Error>>,
    ) = match runtime {
        SupportedRuntime::AssetHubPolkadot => (
            Box::pin(suno_asset_hub_polkadot::fetch_era_data(api, block_hash)),
            Box::pin(suno_asset_hub_polkadot::fetch_validators_era_points(
                api,
                block_hash,
                validator_keys,
            )),
            Box::pin(suno_asset_hub_polkadot::fetch_active_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_polkadot::fetch_total_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_polkadot::fetch_active_nominators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_polkadot::fetch_total_nominators_count(
                api, block_hash,
            )),
        ),
        SupportedRuntime::AssetHubKusama => (
            Box::pin(suno_asset_hub_kusama::fetch_era_data(api, block_hash)),
            Box::pin(suno_asset_hub_kusama::fetch_validators_era_points(
                api,
                block_hash,
                validator_keys,
            )),
            Box::pin(suno_asset_hub_kusama::fetch_active_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_kusama::fetch_total_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_kusama::fetch_active_nominators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_kusama::fetch_total_nominators_count(
                api, block_hash,
            )),
        ),
        SupportedRuntime::AssetHubPaseo => (
            Box::pin(suno_asset_hub_paseo::fetch_era_data(api, block_hash)),
            Box::pin(suno_asset_hub_paseo::fetch_validators_era_points(
                api,
                block_hash,
                validator_keys,
            )),
            Box::pin(suno_asset_hub_paseo::fetch_active_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_paseo::fetch_total_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_paseo::fetch_active_nominators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_paseo::fetch_total_nominators_count(
                api, block_hash,
            )),
        ),
        SupportedRuntime::AssetHubWestend => (
            Box::pin(suno_asset_hub_westend::fetch_era_data(api, block_hash)),
            Box::pin(suno_asset_hub_westend::fetch_validators_era_points(
                api,
                block_hash,
                validator_keys,
            )),
            Box::pin(suno_asset_hub_westend::fetch_active_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_westend::fetch_total_validators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_westend::fetch_active_nominators_count(
                api, block_hash,
            )),
            Box::pin(suno_asset_hub_westend::fetch_total_nominators_count(
                api, block_hash,
            )),
        ),
        _ => {
            error!("Unsupported runtime: {:?}", runtime);
            return Ok(());
        }
    };

    let mut era_data_fut = era_data_fut.fuse();
    let mut validators_era_points_fut = validators_era_points_fut.fuse();
    let mut active_validators_count_fut = active_validators_count_fut.fuse();
    let mut total_validators_count_fut = total_validators_count_fut.fuse();
    let mut active_nominators_count_fut = active_nominators_count_fut.fuse();
    let mut total_nominators_count_fut = total_nominators_count_fut.fuse();

    loop {
        select! {
            era_data_result = era_data_fut => {
                match era_data_result {
                    Ok(era) => {
                        tx.send(Action::Chain(ChainAction::UpdateEra(runtime.clone(), era)))?;
                    }
                    Err(e) => warn!(
                        "Failed to fetch era data: {}",
                        e
                    ),
                }
            }
            validators_era_points_result = validators_era_points_fut => {
                match validators_era_points_result {
                    Ok(points_map) => {
                        for key in validator_keys {
                            if let Some(points) = points_map.get(&key.bytes()).copied() {
                                tx.send(Action::Validator(ValidatorAction::UpdateEraPoints(
                                    key.clone(),
                                    points,
                                )))?;
                            }
                        }
                    }
                    Err(e) => warn!(
                        "Failed to fetch validators_era_points {}",
                        e
                    ),
                }
            }
            active_validators_count_result = active_validators_count_fut => {
                match active_validators_count_result {
                    Ok(count) => {
                        tx.send(Action::Chain(ChainAction::UpdateActiveValidators(
                            runtime.clone(),
                            count,
                        )))?;
                    }
                    Err(e) => warn!(
                        "Failed to fetch active_validators_count: {}",
                        e
                    ),
                }
            }
            total_validators_count_result = total_validators_count_fut => {
                match total_validators_count_result {
                    Ok(count) => {
                        tx.send(Action::Chain(ChainAction::UpdateTotalValidators(
                            runtime.clone(),
                            count,
                        )))?;
                    }
                    Err(e) => warn!(
                        "Failed to fetch total_validators_count: {}",
                        e
                    ),
                }
            }
            active_nominators_count_result = active_nominators_count_fut => {
                match active_nominators_count_result {
                    Ok(count) => {
                        tx.send(Action::Chain(ChainAction::UpdateActiveNominators(
                            runtime.clone(),
                            count,
                        )))?;
                    }
                    Err(e) => warn!(
                        "Failed to fetch active_nominators_count: {}",
                        e
                    ),
                }
            }
            total_nominators_count_result = total_nominators_count_fut => {
                match total_nominators_count_result {
                    Ok(count) => {
                        tx.send(Action::Chain(ChainAction::UpdateTotalNominators(
                            runtime.clone(),
                            count,
                        )))?;
                    }
                    Err(e) => warn!(
                        "Failed to fetch total_nominators_count: {}",
                        e
                    ),
                }
            }
            complete => break
        }
    }

    Ok(())
}

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
            Err(e) => warn!(
                "Failed to fetch identity for {}: {}",
                validator_key.to_string(),
                e
            ),
        }
    }

    Ok(())
}

async fn fetch_and_send_validators_points_from_relay(
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
                    SupportedRuntime::Polkadot => {
                        suno_polkadot::fetch_validator_points(&api, block_hash, &stash).await
                    }
                    SupportedRuntime::Kusama => {
                        suno_kusama::fetch_validator_points(&api, block_hash, &stash).await
                    }
                    SupportedRuntime::Paseo => {
                        suno_paseo::fetch_validator_points(&api, block_hash, &stash).await
                    }
                    SupportedRuntime::Westend => {
                        suno_westend::fetch_validator_points(&api, block_hash, &stash).await
                    }
                    _ => Err(suno_error::Error::from("Unsupported runtime")),
                };
                (validator_key, result)
            }
        })
        .buffer_unordered(CONCURRENT_REQUESTS);

    while let Some((validator_key, result)) = stream.next().await {
        match result {
            Ok(points) => {
                tx.send(Action::Validator(ValidatorAction::UpdatePoints(
                    validator_key,
                    points,
                )))?;
            }
            Err(e) => warn!(
                "Failed to fetch points for {}: {}",
                validator_key.to_string(),
                e
            ),
        }
    }

    Ok(())
}

async fn fetch_and_send_validators_stake_overview(
    api: &OnlineClient<SubstrateConfig>,
    block_hash: H256,
    runtime: &SupportedRuntime,
    era_index: u32,
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
                        suno_asset_hub_polkadot::fetch_validator_stake_overview(
                            &api, block_hash, era_index, &stash,
                        )
                        .await
                    }
                    SupportedRuntime::AssetHubKusama => {
                        suno_asset_hub_kusama::fetch_validator_stake_overview(
                            &api, block_hash, era_index, &stash,
                        )
                        .await
                    }
                    SupportedRuntime::AssetHubPaseo => {
                        suno_asset_hub_paseo::fetch_validator_stake_overview(
                            &api, block_hash, era_index, &stash,
                        )
                        .await
                    }
                    SupportedRuntime::AssetHubWestend => {
                        suno_asset_hub_westend::fetch_validator_stake_overview(
                            &api, block_hash, era_index, &stash,
                        )
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
            Ok(Some(data)) => {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeOverview(
                    validator_key,
                    data,
                )))?;
            }
            Ok(None) => {
                warn!(
                    "No stake overview data found for {}",
                    validator_key.to_string(),
                )
            }
            Err(e) => warn!(
                "Failed to fetch stake overview for {}: {}",
                validator_key.to_string(),
                e
            ),
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
            Err(e) => warn!(
                "Failed to fetch commission for {}: {}",
                validator_key.to_string(),
                e
            ),
        }
    }

    Ok(())
}

async fn fetch_and_send_validators_staking_ledger(
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
                        suno_asset_hub_polkadot::fetch_validator_staking_ledger(
                            &api, block_hash, &stash,
                        )
                        .await
                    }
                    SupportedRuntime::AssetHubKusama => {
                        suno_asset_hub_kusama::fetch_validator_staking_ledger(
                            &api, block_hash, &stash,
                        )
                        .await
                    }
                    SupportedRuntime::AssetHubPaseo => {
                        suno_asset_hub_paseo::fetch_validator_staking_ledger(
                            &api, block_hash, &stash,
                        )
                        .await
                    }
                    SupportedRuntime::AssetHubWestend => {
                        suno_asset_hub_westend::fetch_validator_staking_ledger(
                            &api, block_hash, &stash,
                        )
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
            Ok(Some(data)) => {
                tx.send(Action::Validator(ValidatorAction::UpdateStakeLedger(
                    validator_key.clone(),
                    data,
                )))?;
            }
            Ok(None) => {
                warn!(
                    "No stake ledger data found for {}",
                    validator_key.to_string(),
                )
            }
            Err(e) => warn!(
                "Failed to fetch commission for {}: {}",
                validator_key.to_string(),
                e
            ),
        }
    }

    Ok(())
}
