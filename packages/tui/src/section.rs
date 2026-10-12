use suno_config::Features;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Section {
    #[default]
    Chains,
    Validators,
    Collators,
    /// The main window. Selection moves over every row.
    Main,
}

impl Section {
    pub fn up(&self, features: &Features) -> Self {
        use Section::*;

        match self {
            Chains => Main,
            Main => Self::up_from_main(features),
            Validators => Chains,
            Collators => Self::up_from_collators(features),
        }
    }

    fn up_from_main(features: &Features) -> Self {
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
            Main => Chains,
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
        Self::Main
    }
}
