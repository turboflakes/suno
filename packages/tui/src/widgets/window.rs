use strum::{Display, EnumIter, FromRepr, IntoEnumIterator};
use suno_config::Features;

#[derive(Debug, Clone, Copy, Default, Display, EnumIter, FromRepr, PartialEq, Eq)]
#[strum(serialize_all = "lowercase")]
pub enum Window {
    #[default]
    Validators,
    Collators,
    Logs,
    Help,
}

impl Window {
    /// Windows backed by a feature can be switched off in the config.
    pub fn is_enabled(self, features: &Features) -> bool {
        match self {
            Self::Validators => features.validators_enabled(),
            Self::Collators => features.collators_enabled(),
            Self::Logs | Self::Help => true,
        }
    }

    pub fn is_collators_selected(self) -> bool {
        matches!(self, Self::Collators)
    }

    pub fn is_validators_selected(self) -> bool {
        matches!(self, Self::Validators)
    }

    pub fn is_logs_selected(self) -> bool {
        matches!(self, Self::Logs)
    }

    pub fn is_help_selected(self) -> bool {
        matches!(self, Self::Help)
    }

    /// First window enabled, the one displayed on start.
    pub fn first(features: &Features) -> Self {
        Self::iter()
            .find(|w| w.is_enabled(features))
            .unwrap_or_default()
    }

    /// Next enabled window, wrapping around.
    pub fn next(self, features: &Features) -> Self {
        let count = Self::iter().count();
        (1..=count)
            .filter_map(|step| Self::from_repr((self as usize + step) % count))
            .find(|w| w.is_enabled(features))
            .unwrap_or(self)
    }

    /// Previous enabled window, wrapping around.
    pub fn prev(self, features: &Features) -> Self {
        let count = Self::iter().count();
        (1..=count)
            .filter_map(|step| Self::from_repr((self as usize + count - step) % count))
            .find(|w| w.is_enabled(features))
            .unwrap_or(self)
    }
}
