pub mod error;
pub mod extrinsics;
pub mod storage;
pub mod submit;
pub use storage::{fetch_validator_commission, fetch_validators_era_points};
pub use submit::submit_as_proxy;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/asset_hub_paseo_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
