pub mod constants;
pub mod events;
pub mod extrinsics;
pub mod storage;
pub use events::{handle_events, handle_extrinsic_events};
pub use extrinsics::wrap_call_into_proxy;
pub use storage::{
    fetch_epoch_data, fetch_validator_next_keys, fetch_validator_points,
    fetch_validators_authority_status, fetch_validators_queued_keys,
};

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/paseo_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
