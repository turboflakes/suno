use crate::{babe::Epoch, staking::Era};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    NewEra(Era),
    NewEpoch(Epoch),
}
