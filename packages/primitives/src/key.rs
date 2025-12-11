use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct AccountKey {
    runtime: SupportedRuntime,
    stash: [u8; 32],
}

impl AccountKey {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            runtime,
            stash: *stash.as_ref(),
        }
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.runtime
    }

    pub fn stash(&self) -> AccountId32 {
        AccountId32::from(self.stash)
    }
}
