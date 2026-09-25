use crate::palette::Palette;
use ratatui::style::Color;

// Polkadot Original brand colors: Pink #E6007A (Pantone PMS 213 C), Black and White.

const GRAY_00: Color = Color::Rgb(255, 255, 255); // color_00
const GRAY_01: Color = Color::Rgb(227, 227, 227); // color_01
const GRAY_02: Color = Color::Rgb(198, 198, 198); // color_02
const BLACK: Color = Color::Rgb(0, 0, 0);
const PINK: Color = Color::Rgb(230, 0, 122);

type Name = &'static str;

pub const SUNO_POLKADOT_ORIGINAL_PALETTE: (Name, Palette) = (
    "Polkadot Original",
    Palette {
        color_00: BLACK,
        color_01: BLACK,
        color_02: BLACK,
        color_03: BLACK,
        color_04: GRAY_02,
        color_05: GRAY_01,
        color_06: GRAY_00,
        color_07: GRAY_02,
        color_08: GRAY_01,
        color_09: GRAY_00,
        color_10: PINK,
        color_11: PINK,
        color_12: PINK,
        color_13: PINK,
        color_14: PINK,
        color_15: PINK,
        background: GRAY_00,
        foreground: BLACK,
        cursor_color: GRAY_01,
        cursor_text: BLACK,
        selection_background: BLACK,
        selection_foreground: GRAY_00,
    },
);
