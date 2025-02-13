use crate::config::SupportedRuntime;
use subxt::utils::AccountId32;

/// Common trait for account-related functionality
pub trait AccountDisplay {
    fn stash(&self) -> &AccountId32;

    fn to_compact_string(&self, size: usize) -> String {
        let account_id = self.stash().to_string();
        format!(
            "{}...{}",
            &account_id[..size],
            &account_id[account_id.len() - size..]
        )
    }
}

/// Common struct for shared fields
#[derive(Debug, Clone)]
pub struct NodeAccount {
    pub runtime: SupportedRuntime,
    pub stash: AccountId32,
    pub identity: Option<String>,
}

impl NodeAccount {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            runtime,
            stash,
            identity: None,
        }
    }
}

/// Implement common functionality
impl AccountDisplay for NodeAccount {
    fn stash(&self) -> &AccountId32 {
        &self.stash
    }
}

/// Specific types using composition
#[derive(Debug, Clone)]
pub struct Collator {
    account: NodeAccount,
}

impl Collator {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account: NodeAccount::new(runtime, stash),
        }
    }

    // Getter methods if needed
    pub fn runtime(&self) -> &SupportedRuntime {
        &self.account.runtime
    }

    pub fn identity(&self) -> Option<&String> {
        self.account.identity.as_ref()
    }
}

// Implement the trait for Collator
impl AccountDisplay for Collator {
    fn stash(&self) -> &AccountId32 {
        &self.account.stash
    }
}
