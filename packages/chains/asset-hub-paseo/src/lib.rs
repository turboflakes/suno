pub mod error;
pub mod staking;
pub mod storage;
pub mod submit;
pub use storage::fetch_initial_validator_data;
pub use submit::submit_as_proxy;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/asset_hub_paseo_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
