use crate::display::format_millis;
use subxt::config::substrate::DigestItem;

/// Consensus engine id used by `pallet_babe` to tag its slot-claim pre-runtime digest.
const BABE_ENGINE_ID: [u8; 4] = *b"BABE";

/// Extracts the Babe slot a block was authored for, from its pre-runtime digest.
///
/// The digest payload is a SCALE-encoded `sp_consensus_babe::digests::PreDigest`, whose
/// three variants (`Primary`, `SecondaryPlain`, `SecondaryVRF`) all share the same layout
/// for their first two fields: a 1-byte variant tag, a 4-byte `authority_index`, and an
/// 8-byte `slot`. Any trailing VRF fields are ignored.
pub fn extract_babe_slot(digest_logs: &[DigestItem]) -> Option<u64> {
    digest_logs.iter().find_map(|item| match item {
        DigestItem::PreRuntime(id, data) if *id == BABE_ENGINE_ID => {
            let slot_bytes: [u8; 8] = data.get(5..13)?.try_into().ok()?;
            Some(u64::from_le_bytes(slot_bytes))
        }
        _ => None,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Epoch {
    // Epoch index
    index: u64,
    // Block number that the epoch started
    start_bn: u32,
    // Number of blocks each epoch should take
    duration_bn: u64,
    // Expected average block creation in milliseconds
    block_time_ms: u64,
}

impl Epoch {
    pub fn new(index: u64, start_bn: u32, duration_bn: u64, block_time_ms: u64) -> Self {
        Self {
            index,
            start_bn,
            duration_bn,
            block_time_ms,
        }
    }

    pub fn index(&self) -> u64 {
        self.index
    }

    pub fn start(&self) -> u64 {
        self.start_bn as u64
    }

    pub fn duration(&self) -> u64 {
        self.duration_bn
    }

    pub fn block_time_ms(&self) -> u64 {
        self.block_time_ms
    }

    pub fn progress(&self, current_block_number: u64) -> f64 {
        if current_block_number < self.start() {
            return 0.0;
        }
        let diff = current_block_number - self.start();
        diff as f64 / self.duration() as f64
    }

    pub fn countdown_time(&self, current_block_number: u64) -> String {
        if current_block_number < self.start()
            || current_block_number >= self.start() + self.duration()
        {
            return format_millis(0, true, false);
        }
        let diff = self.duration() - (current_block_number - self.start());
        format_millis(diff * self.block_time_ms, true, false)
    }
}
