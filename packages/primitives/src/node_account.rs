use crate::key::AccountKey;
use ratatui::widgets::Row;
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

/// Common trait for account-related functionality
pub trait AccountDisplay {
    fn stash(&self) -> AccountId32;

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
    account_key: AccountKey,
    identity: Option<String>,
}

impl NodeAccount {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account_key: AccountKey::new(runtime, stash),
            identity: None,
        }
    }

    pub fn account_key(&self) -> &AccountKey {
        &self.account_key
    }

    pub fn runtime(&self) -> &SupportedRuntime {
        &self.account_key.runtime()
    }

    pub fn stash(&self) -> AccountId32 {
        self.account_key.stash()
    }

    pub fn identity(&self) -> &Option<String> {
        &self.identity
    }

    pub fn set_identity(&mut self, identity: String) {
        self.identity = Some(identity);
    }

    pub fn account_format(&self) -> u32 {
        self.runtime().account_format()
    }

    pub fn token_decimals(&self) -> u32 {
        self.runtime().token_decimals()
    }

    pub fn token_symbol(&self) -> String {
        self.runtime().token_symbol()
    }
}

impl AccountDisplay for NodeAccount {
    fn stash(&self) -> AccountId32 {
        self.stash()
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
        &self.account.runtime()
    }

    pub fn identity(&self) -> Option<&String> {
        self.account.identity.as_ref()
    }
}

impl AccountDisplay for Collator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }
}

impl From<&Collator> for Row<'_> {
    fn from(c: &Collator) -> Self {
        let c = c.clone();
        Row::new(vec![c.runtime().to_string(), c.to_compact_string(5)])
    }
}
