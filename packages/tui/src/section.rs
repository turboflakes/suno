use suno_config::Features;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Section {
    #[default]
    Chains,
    Validators,
    Collators,
    /// No section is selected.
    None,
}

impl Section {
    pub fn up(&self, features: &Features) -> Self {
        use Section::*;

        match self {
            Chains => None,
            None => Self::up_from_none(features),
            Validators => Chains,
            Collators => Self::up_from_collators(features),
        }
    }

    fn up_from_none(features: &Features) -> Self {
        if features.collators_enabled() {
            Self::Collators
        } else if features.validators_enabled() {
            Self::Validators
        } else {
            Self::Chains
        }
    }

    fn up_from_collators(features: &Features) -> Self {
        if features.validators_enabled() {
            Self::Validators
        } else {
            Self::Chains
        }
    }

    pub fn down(&self, features: &Features) -> Self {
        use Section::*;

        match self {
            Chains => Self::down_from_chains(features),
            Validators => Self::down_from_validators(features),
            Collators => Self::down_from_collators(),
            None => Chains,
        }
    }

    fn down_from_chains(features: &Features) -> Self {
        if features.validators_enabled() {
            Self::Validators
        } else {
            Self::down_from_validators(features)
        }
    }

    fn down_from_validators(features: &Features) -> Self {
        if features.collators_enabled() {
            Self::Collators
        } else {
            Self::down_from_collators()
        }
    }

    fn down_from_collators() -> Self {
        Self::None
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Section::None)
    }
}
