pub mod borland;
pub mod dracula;
pub mod homebrew;
pub mod monokai_remastered;
pub mod nightrun;
pub mod polkadot_classic;
pub mod suno;
pub mod tokyo_night;

use crate::palette::Palette;

type Name = &'static str;

/// Every builtin theme, in a single place so adding one is a one-line
/// change here instead of a change per consumer.
pub const BUILTIN_PALETTES: &[(Name, Palette)] = &[
    suno::SUNO_DARK_PALETTE,
    suno::SUNO_LIGHT_PALETTE,
    nightrun::SUNO_NIGHTRUN_PALETTE,
    dracula::SUNO_DRACULA_PALETTE,
    borland::SUNO_BORLAND_PALETTE,
    homebrew::SUNO_HOMEBREW_PALETTE,
    monokai_remastered::SUNO_MONOKAI_REMASTERED_PALETTE,
    polkadot_classic::SUNO_POLKADOT_CLASSIC_PALETTE,
    tokyo_night::SUNO_TOKYO_NIGHT_PALETTE,
];
