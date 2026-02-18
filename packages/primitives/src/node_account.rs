use crate::display::to_compact_string;
use crate::{identity::Identity, key::AccountKey};
use ratatui::widgets::Row;
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

/// Common trait for account-related functionality
pub trait AccountDisplay {
    fn stash(&self) -> AccountId32;

    fn to_compact_string(&self, size: usize) -> String {
        to_compact_string(&self.stash(), size)
    }
}

/// Common struct for shared fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAccount {
    account_key: AccountKey,
    identity: Option<Identity>,
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

    pub fn identity(&self) -> &Option<Identity> {
        &self.identity
    }

    pub fn set_identity(&mut self, identity: Option<Identity>) {
        self.identity = identity;
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

    pub fn identity(&self) -> Option<&Identity> {
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

pub fn get_account_id_from_storage_key(bytes: &[u8]) -> AccountId32 {
    let v: [u8; 32] = get_account_bytes_from_storage_key(bytes);
    v.into()
}

pub fn get_account_bytes_from_storage_key(bytes: &[u8]) -> [u8; 32] {
    let s = &bytes[bytes.len() - 32..];
    s.try_into().expect("slice with incorrect length")
}
