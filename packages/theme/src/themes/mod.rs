pub mod borland;
pub mod dracula;
pub mod homebrew;
pub mod monokai_remastered;
pub mod nightrun;
pub mod polkadot_light;
pub mod suno;

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
    polkadot_light::SUNO_POLKADOT_LIGHT_PALETTE,
];
