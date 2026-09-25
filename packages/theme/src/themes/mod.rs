pub mod nightrun;
pub mod suno;

use crate::palette::Palette;

type Name = &'static str;

/// Every builtin theme, in a single place so adding one is a one-line
/// change here instead of a change per consumer.
pub const BUILTIN_PALETTES: &[(Name, Palette)] = &[
    suno::SUNO_DARK_PALETTE,
    suno::SUNO_LIGHT_PALETTE,
    nightrun::SUNO_NIGHTRUN_PALETTE,
];
