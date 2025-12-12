pub mod error;
pub mod storage;
pub use storage::fas_validator_points;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/paseo_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
