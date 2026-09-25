use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the Ghostty "Monokai Remastered" theme:
// https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/ghostty/Monokai%20Remastered

const GRAY_00: Color = Color::Rgb(217, 217, 217); // color_00 - d9d9d9 (foreground)
const GRAY_01: Color = Color::Rgb(194, 194, 194); // color_01
const GRAY_02: Color = Color::Rgb(171, 171, 171); // color_02
const GRAY_03: Color = Color::Rgb(149, 149, 149); // color_03
const GRAY_04: Color = Color::Rgb(126, 126, 126); // color_04
const GRAY_05: Color = Color::Rgb(103, 103, 103); // color_05
const GRAY_06: Color = Color::Rgb(80, 80, 80); // color_06
const GRAY_07: Color = Color::Rgb(58, 58, 58); // color_07
const GRAY_08: Color = Color::Rgb(35, 35, 35); // color_08
const GRAY_09: Color = Color::Rgb(12, 12, 12); // color_09 - 0c0c0c (background)
const PINK_05: Color = Color::Rgb(252, 191, 215); // color_10
const PINK_04: Color = Color::Rgb(250, 128, 175); // color_11
const PINK_03: Color = Color::Rgb(246, 56, 130); // color_12
const PINK_02: Color = Color::Rgb(244, 0, 95); // color_13 - Secondary - f4005f
const PINK_01: Color = Color::Rgb(190, 0, 74); // color_14
const PINK_00: Color = Color::Rgb(134, 0, 52); // color_15

type Name = &'static str;

pub const SUNO_MONOKAI_REMASTERED_PALETTE: (Name, Palette) = (
    "Monokai Remastered",
    Palette {
        color_00: GRAY_00,
        color_01: GRAY_01,
        color_02: GRAY_02,
        color_03: GRAY_03,
        color_04: GRAY_04,
        color_05: GRAY_05,
        color_06: GRAY_06,
        color_07: GRAY_07,
        color_08: GRAY_08,
        color_09: GRAY_09,
        color_10: PINK_05,
        color_11: PINK_04,
        color_12: PINK_03,
        color_13: PINK_02,
        color_14: PINK_01,
        color_15: PINK_00,
        background: GRAY_09,
        foreground: GRAY_01,
        cursor_color: GRAY_08,
        cursor_text: GRAY_01,
        selection_background: GRAY_01,
        selection_foreground: GRAY_09,
    },
);
