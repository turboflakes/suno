use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

pub type AccountBytes = [u8; 32];

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct AccountKey {
    pub runtime: SupportedRuntime,
    pub bytes: AccountBytes,
}

impl AccountKey {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            runtime,
            bytes: *stash.as_ref(),
        }
    }

    pub fn from_bytes(runtime: SupportedRuntime, bytes: AccountBytes) -> Self {
        Self { runtime, bytes }
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.runtime
    }

    pub fn bytes(&self) -> AccountBytes {
        self.bytes
    }

    pub fn stash(&self) -> AccountId32 {
        AccountId32::from(self.bytes)
    }
}

impl std::fmt::Display for AccountKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "account {} at {}", self.stash(), self.runtime())
    }
}
