use suno_config::Features;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Section {
    #[default]
    Chains,
    Validators,
    Collators,
    Nominators,
    Proxies,
    /// No section is selected.
    None,
}

impl Section {
    /// The sections that can be selected, from top to bottom.
    fn selectable(features: &Features, proxies_enabled: bool) -> Vec<Self> {
        let mut sections = vec![Self::Chains];
        if features.validators_enabled() {
            sections.push(Self::Validators);
        }
        if features.collators_enabled() {
            sections.push(Self::Collators);
        }
        if features.nominators_enabled() {
            sections.push(Self::Nominators);
        }
        if proxies_enabled {
            sections.push(Self::Proxies);
        }
        sections
    }

    pub fn up(&self, features: &Features, proxies_enabled: bool) -> Self {
        let sections = Self::selectable(features, proxies_enabled);
        match self {
            Self::Chains => Self::None,
            // A section that can't be selected anymore, e.g. the proxies once disabled, goes
            // back to the last one that can
            _ => match sections.iter().position(|s| s == self) {
                Some(index) if index > 0 => sections[index - 1].clone(),
                _ => sections.last().cloned().unwrap_or(Self::Chains),
            },
        }
    }

    pub fn down(&self, features: &Features, proxies_enabled: bool) -> Self {
        let sections = Self::selectable(features, proxies_enabled);
        match self {
            Self::None => Self::Chains,
            _ => match sections.iter().position(|s| s == self) {
                Some(index) => sections.get(index + 1).cloned().unwrap_or(Self::None),
                None => Self::None,
            },
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Section::None)
    }
}
