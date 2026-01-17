pub mod storage;
pub use storage::fetch_identity;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/people_westend_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
