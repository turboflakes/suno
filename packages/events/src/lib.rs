use sp_arithmetic::Permill;
use suno_primitives::{babe::Epoch, staking::Era};

/// Application asynchronous events supported.
#[derive(Debug)]
pub enum Event {
    NewEra(Era),
    NewEpoch(Epoch),
    TotalStakedFetched(Permill),
}
