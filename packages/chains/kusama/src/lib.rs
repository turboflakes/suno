pub mod blocks;
pub mod constants;
pub mod extrinsics;
pub mod runtime_apis;
pub mod storage;
pub mod utils;

pub use blocks::{process_block_extrinsics, process_runtime_events, process_transaction_events};
pub use extrinsics::wrap_call_into_proxy;
pub use runtime_apis::fetch_metadata;
pub use storage::{
    fetch_and_validate_proxy_account, fetch_epoch_data, fetch_validator_next_keys,
    fetch_validator_points, fetch_validators_authority_status, fetch_validators_queued_keys,
};

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/kusama_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
