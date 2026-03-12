use crate::balance::{Amount, Balance};
use crate::display::{format_planks, to_compact_string};
use crate::{identity::Identity, key::AccountKey};
use ratatui::widgets::Row;
use subxt::utils::AccountId32;
use suno_config::SupportedRuntime;

/// Common trait for account-related functionality
pub trait AccountDisplay {
    fn stash(&self) -> AccountId32;
    fn account_format(&self) -> u16;

    fn to_compact_string(&self, size: usize) -> String {
        to_compact_string(&self.stash(), self.account_format(), size)
    }
}

/// Common struct for shared fields
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAccount {
    account_key: AccountKey,
    identity: Option<Identity>,
    balance: Balance,
}

impl NodeAccount {
    pub fn new(runtime: SupportedRuntime, stash: AccountId32) -> Self {
        Self {
            account_key: AccountKey::new(runtime, stash),
            identity: None,
            balance: Balance::default(),
        }
    }

    pub fn account_key(&self) -> &AccountKey {
        &self.account_key
    }

    pub fn runtime(&self) -> SupportedRuntime {
        self.account_key.runtime()
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

    pub fn set_balance(&mut self, balance: Balance) {
        self.balance = balance;
    }

    pub fn add_free_amount(&mut self, amount: Amount) {
        self.balance.add_free_amount(amount);
    }

    pub fn account_format(&self) -> u16 {
        self.runtime().account_format()
    }

    pub fn token_decimals(&self) -> u32 {
        self.runtime().token_decimals()
    }

    pub fn token_symbol(&self) -> &'static str {
        self.runtime().token_symbol()
    }

    pub fn free_balance(&self) -> u128 {
        self.balance.free_balance()
    }

    pub fn free_balance_as_str(&self, decimal_places: usize) -> String {
        let value = format_planks(self.free_balance(), self.token_decimals(), decimal_places);
        format!("{}{}", value, self.token_symbol())
    }
}

impl AccountDisplay for NodeAccount {
    fn stash(&self) -> AccountId32 {
        self.stash()
    }

    fn account_format(&self) -> u16 {
        self.account_format()
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
    pub fn runtime(&self) -> SupportedRuntime {
        self.account.runtime()
    }

    pub fn identity(&self) -> Option<&Identity> {
        self.account.identity.as_ref()
    }
}

impl AccountDisplay for Collator {
    fn stash(&self) -> AccountId32 {
        self.account.stash()
    }

    fn account_format(&self) -> u16 {
        self.account.account_format()
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
