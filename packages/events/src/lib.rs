use sp_arithmetic::Permill;
use suno_primitives::{babe::Epoch, staking::Era, validator::ValidatorStatus};

type AccountBytes = [u8; 32];

/// Application asynchronous events supported.
#[derive(Debug)]
pub enum Event {
    NewEra(Era),
    NewEpoch(Epoch),
    TotalStaked(Permill),
    AuthorityStatus(AccountBytes, ValidatorStatus),
}
