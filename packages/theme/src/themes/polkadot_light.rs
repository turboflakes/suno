use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the light color scheme (`:root`) of https://polkadot.com

const GRAY_00: Color = Color::Rgb(28, 25, 23); // color_00, color_01, color_03, color_14, color_15, selection_background - #1c1917
const GRAY_01: Color = Color::Rgb(68, 64, 60); // color_13 - #44403c
const GRAY_02: Color = Color::Rgb(87, 83, 78); // color_02, color_04 - #57534e
const GRAY_07: Color = Color::Rgb(235, 235, 234); // color_05, color_07 - #ebebea
const GRAY_08: Color = Color::Rgb(245, 245, 244); // color_06, color_08, cursor_color - #f5f5f4
const GRAY_09: Color = Color::Rgb(250, 250, 249); // color_09, selection_foreground - #fafaf9

type Name = &'static str;

pub const SUNO_POLKADOT_LIGHT_PALETTE: (Name, Palette) = (
    "Polkadot Light",
    Palette {
        color_00: GRAY_00,
        color_01: GRAY_00,
        color_02: GRAY_02,
        color_03: GRAY_00,
        color_04: GRAY_02,
        color_05: GRAY_07,
        color_06: GRAY_08,
        color_07: GRAY_07,
        color_08: GRAY_08,
        color_09: GRAY_09,
        color_10: GRAY_00,
        color_11: GRAY_00,
        color_12: GRAY_01,
        color_13: GRAY_01,
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
