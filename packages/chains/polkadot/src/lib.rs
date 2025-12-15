pub mod error;
pub mod storage;
pub use storage::fetch_validator_points;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/polkadot_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
