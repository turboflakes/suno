use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the dark color scheme (`.dark`) of https://polkadot.com

const GRAY_00: Color = Color::Rgb(245, 245, 244); // color_00, color_01, color_03, color_14, color_15, selection_background - #f5f5f4
const GRAY_02: Color = Color::Rgb(214, 211, 209); // color_02, color_12, color_13 - #d6d3d1
const GRAY_04: Color = Color::Rgb(168, 162, 158); // color_04 - #a8a29e
const GRAY_07: Color = Color::Rgb(28, 25, 23); // color_05, color_07 - #1c1917
const GRAY_08: Color = Color::Rgb(41, 37, 36); // color_06, color_08, cursor_color - #292524
const GRAY_09: Color = Color::Rgb(15, 15, 15); // color_09, selection_foreground - #0f0f0f

type Name = &'static str;

pub const SUNO_POLKADOT_DARK_PALETTE: (Name, Palette) = (
    "Polkadot Dark",
    Palette {
        color_00: GRAY_00,
        color_01: GRAY_00,
        color_02: GRAY_02,
        color_03: GRAY_00,
        color_04: GRAY_04,
        color_05: GRAY_07,
        color_06: GRAY_08,
        color_07: GRAY_07,
        color_08: GRAY_08,
        color_09: GRAY_09,
        color_10: GRAY_00,
        color_11: GRAY_00,
        color_12: GRAY_02,
        color_13: GRAY_02,
        color_14: GRAY_00,
        color_15: GRAY_00,
        background: GRAY_09,
        foreground: GRAY_00,
        cursor_color: GRAY_08,
        cursor_text: GRAY_00,
        selection_background: GRAY_00,
        selection_foreground: GRAY_09,
    },
);
