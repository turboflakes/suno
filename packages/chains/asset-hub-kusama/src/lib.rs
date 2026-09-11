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
    runtime_metadata_path = "artifacts/metadata/asset_hub_kusama_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}

#[cfg(test)]
mod tests {
    use suno_config::transactions::should_use_v5_transaction;

    /// Guards the V4/V5 transaction-format decision in
    /// `suno_config::transactions::sign_and_submit_then_watch`: whether this network's
    /// committed metadata advertises a second transaction-extension pipeline (which is
    /// what makes a V5 "General" transaction able to carry a signed origin here). If a
    /// metadata refresh flips this, the signing path for this network changes too, so it
    /// deserves a deliberate look rather than a silent behavior change.
    #[test]
    fn should_use_v5_transaction_matches_expectations() {
        let bytes = include_bytes!("../artifacts/metadata/asset_hub_kusama_metadata_small.scale");
        let metadata = subxt::Metadata::decode_from(&bytes[..]).expect("valid metadata");
        assert!(!should_use_v5_transaction(&metadata));
    }
}
