mod error;
mod update;

pub use crate::error::Error;
pub use crate::update::{check_for_update, run};
