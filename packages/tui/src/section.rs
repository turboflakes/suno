use snops_config::Features;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Section {
    #[default]
    Chains,
    Validators,
    Collators,
    Rpcs,
}

impl Section {
    pub fn up(&self, features: &Features) -> Self {
        use Section::*;

        match self {
            Chains => Self::up_from_chains(features),
            Validators => Chains,
            Collators => Self::up_from_collators(features),
            Rpcs => Self::up_from_rpcs(features),
        }
    }

    fn up_from_chains(features: &Features) -> Self {
        if features.enable_rpcs {
            Self::Rpcs
        } else if features.enable_collators {
            Self::Collators
        } else {
            Self::Validators
        }
    }

    fn up_from_collators(features: &Features) -> Self {
        if features.enable_validators {
            Self::Validators
        } else {
            Self::Chains
        }
    }

    fn up_from_rpcs(features: &Features) -> Self {
        if features.enable_collators {
            Self::Collators
        } else if features.enable_validators {
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
            Collators => Self::down_from_collators(features),
            Rpcs => Chains,
        }
    }

    fn down_from_chains(features: &Features) -> Self {
        if features.enable_validators {
            Self::Validators
        } else if features.enable_collators {
            Self::Collators
        } else {
            Self::Rpcs
        }
    }

    fn down_from_validators(features: &Features) -> Self {
        if features.enable_collators {
            Self::Collators
        } else if features.enable_rpcs {
            Self::Rpcs
        } else {
            Self::Chains
        }
    }

    fn down_from_collators(features: &Features) -> Self {
        if features.enable_rpcs {
            Self::Rpcs
        } else {
            Self::Chains
        }
    }
}
