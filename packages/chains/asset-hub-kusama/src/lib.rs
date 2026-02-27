pub mod constants;
pub mod events;
pub mod extrinsics;
pub mod storage;
pub mod submit;
pub use events::handle_events;
pub use storage::{
    fetch_active_nominators_count, fetch_active_validators_count, fetch_era_data,
    fetch_total_nominators_count, fetch_total_staked, fetch_total_validators_count,
    fetch_validator_prefs, fetch_validator_prefs_next, fetch_validator_stake_overview,
    fetch_validator_staking_ledger, fetch_validators_era_points,
};
pub use submit::submit_as_proxy;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/asset_hub_kusama_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
