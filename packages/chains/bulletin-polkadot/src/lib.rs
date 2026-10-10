pub mod blocks;
pub mod constants;
pub mod runtime_apis;
pub mod storage;

pub use blocks::process_runtime_events;
pub use constants::fetch_slot_duration;
pub use runtime_apis::fetch_metadata;
pub use storage::{
    fetch_aura_authorities, fetch_balance, fetch_candidate_list,
    fetch_collator_last_authored_block, fetch_collator_next_keys, fetch_collators_queued_keys,
    fetch_invulnerables, fetch_session_index, fetch_session_validators,
};

#[subxt::subxt(
    runtime_metadata_path = "artifacts/metadata/bulletin_polkadot_metadata_small.scale",
    derive_for_all_types = "PartialEq, Clone"
)]
mod node_runtime {}
