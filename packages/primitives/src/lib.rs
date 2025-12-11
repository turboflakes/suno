pub mod key;
pub mod node_account;

pub use node_account::{AccountDisplay, Collator, NodeAccount};

pub type ValidatorKey = key::AccountKey;
