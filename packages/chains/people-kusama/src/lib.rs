pub mod error;
pub mod storage;
pub use storage::fetch_display_name;

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/people_kusama_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
