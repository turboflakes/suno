pub mod blocks;
pub mod constants;
pub mod extrinsics;
pub mod runtime_apis;
pub mod storage;

pub use blocks::{process_block_extrinsics, process_runtime_events, process_transaction_events};
pub use extrinsics::wrap_call_into_proxy;
pub use runtime_apis::fetch_metadata;
pub use storage::{
    fetch_active_nominators_count, fetch_active_validators_count, fetch_and_validate_proxy_account,
    fetch_balance, fetch_era_data, fetch_total_nominators_count, fetch_total_staked,
    fetch_total_validators_count, fetch_validator_payee, fetch_validator_prefs,
    fetch_validator_prefs_next, fetch_validator_stake_overview, fetch_validator_staking_ledger,
    fetch_validators_era_points,
};
pub mod utils;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/asset_hub_westend_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
