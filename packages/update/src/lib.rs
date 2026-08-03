mod error;
pub mod update;

pub use crate::error::Error;
pub use crate::update::{check_for_update, run_update, AssetName, Checksum, Release};
