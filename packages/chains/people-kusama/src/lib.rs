pub mod runtime_apis;
pub mod storage;

pub use runtime_apis::fetch_metadata;
pub use storage::fetch_identity;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/people_kusama_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
