pub mod staking;
pub mod submit;
pub use submit::submit_as_proxy;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/westend_metadata.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
