use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// What a feature applies to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FeatureScope {
    General,
    Validators,
    Collators,
}

impl std::fmt::Display for FeatureScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::General => write!(f, "general"),
            Self::Validators => write!(f, "validators"),
            Self::Collators => write!(f, "collators"),
        }
    }
}

/// Every feature, in the order they are listed.
pub const ALL_FEATURES: [Feature; 14] = [
    Feature::EnableCheckUpdate,
    Feature::EnableValidators,
    Feature::ShowAllValidators,
    Feature::ShowNominatorsCounter,
    Feature::ShowNominatorsStake,
    Feature::ShowOwnStake,
    Feature::ShowCommission,
    Feature::ShowPayee,
    Feature::ShowKeys,
    Feature::EnableCollators,
    Feature::ShowAllCollators,
    Feature::ShowDeposit,
    Feature::ShowLastBlock,
    Feature::ShowNextSlot,
];

/// The features that can be switched on and off, from the config file or from the TUI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Feature {
    EnableCheckUpdate,
    EnableValidators,
    ShowAllValidators,
    ShowNominatorsCounter,
    ShowNominatorsStake,
    ShowOwnStake,
    ShowCommission,
    ShowPayee,
    ShowKeys,
    EnableCollators,
    ShowAllCollators,
    ShowDeposit,
    ShowLastBlock,
    ShowNextSlot,
}

impl Feature {
    /// The name of the feature in the config file.
    pub fn key(&self) -> &'static str {
        match self {
            Self::EnableCheckUpdate => "enable_check_update",
            Self::EnableValidators => "enable_validators",
            Self::ShowAllValidators => "show_all_validators",
            Self::ShowNominatorsCounter => "show_nominators_counter",
            Self::ShowNominatorsStake => "show_nominators_stake",
            Self::ShowOwnStake => "show_own_stake",
            Self::ShowCommission => "show_commission",
            Self::ShowPayee => "show_payee",
            Self::ShowKeys => "show_keys",
            Self::EnableCollators => "enable_collators",
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

    pub fn scope(&self) -> FeatureScope {
        match self {
            Self::EnableCheckUpdate => FeatureScope::General,
            Self::EnableValidators
            | Self::ShowAllValidators
            | Self::ShowNominatorsCounter
            | Self::ShowNominatorsStake
            | Self::ShowOwnStake
            | Self::ShowCommission
            | Self::ShowPayee => FeatureScope::Validators,
            // The session keys are shown for the validators and the collators
            Self::ShowKeys => FeatureScope::General,
            Self::EnableCollators
            | Self::ShowAllCollators
            | Self::ShowDeposit
            | Self::ShowLastBlock
            | Self::ShowNextSlot => FeatureScope::Collators,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::EnableCheckUpdate => "check for a new version on start",
            Self::EnableValidators => "enable the validators window and its data",
            Self::ShowAllValidators => "show every on-chain validator, active or waiting",
            Self::ShowNominatorsCounter => "show the number of nominators",
            Self::ShowNominatorsStake => "show the stake of the nominators",
            Self::ShowOwnStake => "show the stake of the validator",
            Self::ShowCommission => "show the validator commission",
            Self::ShowPayee => "show the reward destination",
            Self::ShowKeys => "show the session keys",
            Self::EnableCollators => "enable the collators window and its data",
            Self::ShowAllCollators => "show every on-chain collator",
            Self::ShowDeposit => "show the deposit of the collators",
            Self::ShowLastBlock => "show the last produced block and how long ago",
            Self::ShowNextSlot => "show the next slot number and blocks produced in the slot",
        }
    }

    /// Whether a change only applies once the app is started again.
    pub fn needs_restart(&self) -> bool {
        matches!(
            self,
            Self::EnableCheckUpdate | Self::EnableValidators | Self::EnableCollators
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
    #[serde(default)]
    enable_rpcs: AtomicBool,
    #[serde(default = "enabled")]
    show_all_validators: AtomicBool,
    #[serde(default = "enabled")]
    show_all_collators: AtomicBool,
    #[serde(default = "enabled")]
    enable_check_update: AtomicBool,
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
            enable_rpcs: AtomicBool::new(false),
            show_all_validators: enabled(),
            show_all_collators: enabled(),
            enable_check_update: enabled(),
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
            Feature::EnableCheckUpdate => &self.enable_check_update,
            Feature::EnableValidators => &self.enable_validators,
            Feature::ShowAllValidators => &self.show_all_validators,
            Feature::ShowNominatorsCounter => &self.show_nominators_counter,
            Feature::ShowNominatorsStake => &self.show_nominators_stake,
            Feature::ShowOwnStake => &self.show_own_stake,
            Feature::ShowCommission => &self.show_commission,
            Feature::ShowPayee => &self.show_payee,
            Feature::ShowKeys => &self.show_keys,
            Feature::EnableCollators => &self.enable_collators,
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

    pub fn rpcs_enabled(&self) -> bool {
        self.enable_rpcs.load(Ordering::Relaxed)
    }

    pub fn check_update_enabled(&self) -> bool {
        self.get(Feature::EnableCheckUpdate)
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
    }
}
