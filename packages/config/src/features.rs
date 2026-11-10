use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// What a feature applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureScope {
    General,
    Validators,
    Collators,
    Nominators,
}

impl std::fmt::Display for FeatureScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General => write!(f, "general"),
            Self::Validators => write!(f, "validators"),
            Self::Collators => write!(f, "collators"),
            Self::Nominators => write!(f, "nominators"),
        }
    }
}

impl FeatureScope {
    /// The short name, used when a feature has more than one scope.
    pub fn short(&self) -> &'static str {
        match self {
            Self::General => "gen",
            Self::Validators => "val",
            Self::Collators => "col",
            Self::Nominators => "nom",
        }
    }
}

/// Every feature, in the order they are listed.
pub const ALL_FEATURES: [Feature; 17] = [
    Feature::EnableValidators,
    Feature::EnableNominators,
    Feature::EnableCollators,
    Feature::EnableProxies,
    Feature::EnableCheckUpdate,
    Feature::EnableLightclientDb,
    Feature::ShowAllValidators,
    Feature::ShowNominatorsCounter,
    Feature::ShowNominatorsStake,
    Feature::ShowOwnStake,
    Feature::ShowCommission,
    Feature::ShowPayee,
    Feature::ShowKeys,
    Feature::ShowAllCollators,
    Feature::ShowDeposit,
    Feature::ShowLastBlock,
    Feature::ShowNextSlot,
];

/// The features that can be switched on and off, from the config file or from the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Feature {
    EnableValidators,
    EnableNominators,
    EnableCollators,
    EnableProxies,
    EnableCheckUpdate,
    EnableLightclientDb,
    ShowAllValidators,
    ShowNominatorsCounter,
    ShowNominatorsStake,
    ShowOwnStake,
    ShowCommission,
    ShowPayee,
    ShowKeys,
    ShowAllCollators,
    ShowDeposit,
    ShowLastBlock,
    ShowNextSlot,
}

impl Feature {
    /// The name of the feature in the config file.
    pub fn key(&self) -> &'static str {
        match self {
            Self::EnableValidators => "enable_validators",
            Self::EnableNominators => "enable_nominators",
            Self::EnableCollators => "enable_collators",
            Self::EnableProxies => "enable_proxies",
            Self::EnableCheckUpdate => "enable_check_update",
            Self::EnableLightclientDb => "enable_lightclient_db",
            Self::ShowAllValidators => "show_all_validators",
            Self::ShowNominatorsCounter => "show_nominators_counter",
            Self::ShowNominatorsStake => "show_nominators_stake",
            Self::ShowOwnStake => "show_own_stake",
            Self::ShowCommission => "show_commission",
            Self::ShowPayee => "show_payee",
            Self::ShowKeys => "show_keys",
            Self::ShowAllCollators => "show_all_collators",
            Self::ShowDeposit => "show_deposit",
            Self::ShowLastBlock => "show_last_block",
            Self::ShowNextSlot => "show_next_slot",
        }
    }

    pub fn from_key(key: &str) -> Option<Self> {
        ALL_FEATURES
            .into_iter()
            .find(|feature| feature.key() == key)
    }

    /// What the feature applies to, a feature can apply to more than one.
    pub fn scopes(&self) -> &'static [FeatureScope] {
        use FeatureScope::*;
        match self {
            Self::EnableValidators
            | Self::EnableNominators
            | Self::EnableCollators
            | Self::EnableProxies
            | Self::EnableCheckUpdate
            | Self::EnableLightclientDb => &[General],
            Self::ShowAllValidators
            | Self::ShowNominatorsCounter
            | Self::ShowNominatorsStake
            | Self::ShowOwnStake
            | Self::ShowCommission => &[Validators],
            Self::ShowPayee => &[Validators, Nominators],
            Self::ShowKeys => &[Validators, Collators],
            Self::ShowAllCollators
            | Self::ShowDeposit
            | Self::ShowLastBlock
            | Self::ShowNextSlot => &[Collators],
        }
    }

    /// The scopes of the feature, to be displayed.
    pub fn scopes_label(&self) -> String {
        match self.scopes() {
            [scope] => scope.to_string(),
            scopes => scopes
                .iter()
                .map(|scope| scope.short())
                .collect::<Vec<_>>()
                .join(","),
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::EnableValidators => "Enable the validators window and its data",
            Self::EnableNominators => "Enable the nominators section and its data",
            Self::EnableCollators => "Enable the collators window and its data",
            Self::EnableProxies => "Enable the proxies section",
            Self::EnableCheckUpdate => "Check for new versions on startup",
            Self::EnableLightclientDb => {
                "Store the light client database on disk to speed up startup"
            }
            Self::ShowAllValidators => "Display all on-chain validators, active or waiting",
            Self::ShowNominatorsCounter => "Display the number of nominators",
            Self::ShowNominatorsStake => "Display the nominator's total stake",
            Self::ShowOwnStake => "Display the validator's own stake",
            Self::ShowCommission => "Display the validator's commission",
            Self::ShowPayee => "Display the reward destination",
            Self::ShowKeys => "Display the session keys",
            Self::ShowAllCollators => "Display all on-chain collators",
            Self::ShowDeposit => "Display the collator's deposit",
            Self::ShowLastBlock => {
                "Display the last produced block and how long ago it was produced"
            }
            Self::ShowNextSlot => {
                "Display the next slot number and the blocks produced in that slot"
            }
        }
    }

    /// Whether a change only applies once the app is started again.
    pub fn needs_restart(&self) -> bool {
        matches!(
            self,
            Self::EnableValidators
                | Self::EnableNominators
                | Self::EnableCollators
                | Self::EnableProxies
                | Self::EnableCheckUpdate
                | Self::EnableLightclientDb
        )
    }
}

fn enabled() -> AtomicBool {
    AtomicBool::new(true)
}

/// The features, which can be changed while the app runs, e.g. from the TUI.
#[derive(Debug, Serialize, Deserialize)]
pub struct Features {
    #[serde(default = "enabled")]
    enable_validators: AtomicBool,
    #[serde(default = "enabled")]
    enable_collators: AtomicBool,
    #[serde(default = "enabled")]
    enable_nominators: AtomicBool,
    #[serde(default = "enabled")]
    enable_proxies: AtomicBool,
    #[serde(default = "enabled")]
    enable_check_update: AtomicBool,
    #[serde(default = "enabled")]
    enable_lightclient_db: AtomicBool,
    #[serde(default)]
    enable_rpcs: AtomicBool,
    #[serde(default = "enabled")]
    show_all_validators: AtomicBool,
    #[serde(default = "enabled")]
    show_all_collators: AtomicBool,
    #[serde(default = "enabled")]
    show_nominators_counter: AtomicBool,
    #[serde(default = "enabled")]
    show_nominators_stake: AtomicBool,
    #[serde(default = "enabled")]
    show_own_stake: AtomicBool,
    #[serde(default)]
    show_commission: AtomicBool,
    #[serde(default = "enabled")]
    show_payee: AtomicBool,
    #[serde(default = "enabled")]
    show_keys: AtomicBool,
    #[serde(default = "enabled")]
    show_deposit: AtomicBool,
    #[serde(default = "enabled")]
    show_last_block: AtomicBool,
    #[serde(default = "enabled")]
    show_next_slot: AtomicBool,
}

impl Default for Features {
    fn default() -> Self {
        Self {
            enable_validators: enabled(),
            enable_collators: enabled(),
            enable_nominators: enabled(),
            enable_proxies: enabled(),
            enable_check_update: enabled(),
            enable_lightclient_db: enabled(),
            enable_rpcs: AtomicBool::new(false),
            show_all_validators: enabled(),
            show_all_collators: enabled(),
            show_nominators_counter: enabled(),
            show_nominators_stake: enabled(),
            show_own_stake: enabled(),
            show_commission: AtomicBool::new(false),
            show_payee: enabled(),
            show_keys: enabled(),
            show_deposit: enabled(),
            show_last_block: enabled(),
            show_next_slot: enabled(),
        }
    }
}

impl Features {
    pub fn new() -> Self {
        Self::default()
    }

    fn flag(&self, feature: Feature) -> &AtomicBool {
        match feature {
            Feature::EnableValidators => &self.enable_validators,
            Feature::EnableNominators => &self.enable_nominators,
            Feature::EnableCollators => &self.enable_collators,
            Feature::EnableProxies => &self.enable_proxies,
            Feature::EnableCheckUpdate => &self.enable_check_update,
            Feature::EnableLightclientDb => &self.enable_lightclient_db,
            Feature::ShowAllValidators => &self.show_all_validators,
            Feature::ShowNominatorsCounter => &self.show_nominators_counter,
            Feature::ShowNominatorsStake => &self.show_nominators_stake,
            Feature::ShowOwnStake => &self.show_own_stake,
            Feature::ShowCommission => &self.show_commission,
            Feature::ShowPayee => &self.show_payee,
            Feature::ShowKeys => &self.show_keys,
            Feature::ShowAllCollators => &self.show_all_collators,
            Feature::ShowDeposit => &self.show_deposit,
            Feature::ShowLastBlock => &self.show_last_block,
            Feature::ShowNextSlot => &self.show_next_slot,
        }
    }

    pub fn get(&self, feature: Feature) -> bool {
        self.flag(feature).load(Ordering::Relaxed)
    }

    pub fn set(&self, feature: Feature, value: bool) {
        self.flag(feature).store(value, Ordering::Relaxed);
    }

    /// The value of every feature.
    pub fn values(&self) -> Vec<(Feature, bool)> {
        ALL_FEATURES
            .into_iter()
            .map(|feature| (feature, self.get(feature)))
            .collect()
    }

    pub fn validators_enabled(&self) -> bool {
        self.get(Feature::EnableValidators)
    }

    pub fn collators_enabled(&self) -> bool {
        self.get(Feature::EnableCollators)
    }

    pub fn nominators_enabled(&self) -> bool {
        self.get(Feature::EnableNominators)
    }

    pub fn proxies_enabled(&self) -> bool {
        self.get(Feature::EnableProxies)
    }

    pub fn rpcs_enabled(&self) -> bool {
        self.enable_rpcs.load(Ordering::Relaxed)
    }

    pub fn check_update_enabled(&self) -> bool {
        self.get(Feature::EnableCheckUpdate)
    }

    pub fn lightclient_db_enabled(&self) -> bool {
        self.get(Feature::EnableLightclientDb)
    }

    /// Every on-chain collator is listed, only if the collators are enabled.
    pub fn all_collators_visible(&self) -> bool {
        self.collators_enabled() && self.get(Feature::ShowAllCollators)
    }

    /// Every on-chain validator is listed, only if the validators are enabled.
    pub fn all_validators_visible(&self) -> bool {
        self.validators_enabled() && self.get(Feature::ShowAllValidators)
    }

    pub fn nominators_counter_visible(&self) -> bool {
        self.get(Feature::ShowNominatorsCounter)
    }

    pub fn nominators_stake_visible(&self) -> bool {
        self.get(Feature::ShowNominatorsStake)
    }

    pub fn own_stake_visible(&self) -> bool {
        self.get(Feature::ShowOwnStake)
    }

    pub fn commission_visible(&self) -> bool {
        self.get(Feature::ShowCommission)
    }

    pub fn payee_visible(&self) -> bool {
        self.get(Feature::ShowPayee)
    }

    pub fn keys_visible(&self) -> bool {
        self.get(Feature::ShowKeys)
    }

    pub fn deposit_visible(&self) -> bool {
        self.get(Feature::ShowDeposit)
    }

    pub fn last_block_visible(&self) -> bool {
        self.get(Feature::ShowLastBlock)
    }

    pub fn next_slot_visible(&self) -> bool {
        self.get(Feature::ShowNextSlot)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_short_scope_names_are_only_used_with_multiple_scopes() {
        assert_eq!(Feature::ShowCommission.scopes_label(), "validators");
        assert_eq!(Feature::ShowPayee.scopes_label(), "val, nom");
        assert_eq!(Feature::ShowKeys.scopes_label(), "val, col");
    }

    #[test]
    fn every_feature_has_its_own_key() {
        for feature in ALL_FEATURES {
            assert_eq!(Feature::from_key(feature.key()), Some(feature));
        }
    }

    #[test]
    fn a_feature_can_be_changed_while_the_app_runs() {
        let features = Features::default();
        assert!(features.payee_visible());
        features.set(Feature::ShowPayee, false);
        assert!(!features.payee_visible());
        assert!(features.values().contains(&(Feature::ShowPayee, false)));
    }

    #[test]
    fn features_are_read_from_yaml_with_defaults() {
        let features: Features =
            serde_yaml::from_str("show_commission: true\nshow_payee: false\n").unwrap();
        assert!(features.commission_visible());
        assert!(!features.payee_visible());
        assert!(features.validators_enabled());
        assert!(features.lightclient_db_enabled());
    }

    #[test]
    fn the_nominators_are_enabled_unless_the_user_opts_out() {
        assert!(Features::default().nominators_enabled());
        let features: Features = serde_yaml::from_str("enable_nominators: false\n").unwrap();
        assert!(!features.nominators_enabled());
        assert!(features
            .values()
            .contains(&(Feature::EnableNominators, false)));
    }

    #[test]
    fn the_lightclient_db_can_be_opted_out() {
        let features: Features = serde_yaml::from_str("enable_lightclient_db: false\n").unwrap();
        assert!(!features.lightclient_db_enabled());
    }
}
