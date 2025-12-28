use suno_config::SupportedRuntime;

#[derive(Debug, Clone)]
pub struct Staking {
    runtime: SupportedRuntime,
    era: u32,
    session: u32,
    nominators_count: u32,
    validators_count: u32,
}

impl Staking {
    pub fn new(runtime: SupportedRuntime) -> Self {
        Self {
            runtime,
            era: 0,
            session: 0,
            nominators_count: 0,
            validators_count: 0,
        }
    }

    pub fn key(&self) -> &SupportedRuntime {
        &self.runtime
    }

    pub fn name(&self) -> &str {
        &self.runtime.as_str()
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.runtime
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Default)]
pub struct StakeOverview {
    pub own: u128,
    pub total: u128,
    pub nominators_count: u32,
}

impl StakeOverview {
    pub fn new(own: u128, total: u128, nominators_count: u32) -> Self {
        Self {
            own,
            total,
            nominators_count,
        }
    }

    pub fn own(&self) -> u128 {
        self.own
    }

    pub fn total(&self) -> u128 {
        self.total
    }

    pub fn nominators_count(&self) -> u32 {
        self.nominators_count
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct Unlocking {
    pub era: u32,
    pub value: u128,
}

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct StakeLedger {
    pub active: u128,
    pub total: u128,
    pub unlocking: Vec<Unlocking>,
}

impl StakeLedger {
    // TODO: add unlocking: Vec<Unlocking>
    pub fn new(active: u128, total: u128) -> Self {
        Self {
            active,
            total,
            unlocking: Vec::new(),
        }
    }

    pub fn active(&self) -> u128 {
        self.active
    }

    pub fn total(&self) -> u128 {
        self.total
    }
}
