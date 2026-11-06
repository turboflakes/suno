use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the Ghostty "Monokai Remastered" theme:
// https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/ghostty/Monokai%20Remastered

const GRAY_00: Color = Color::Rgb(217, 217, 217); // color_00 - #d9d9d9 (foreground)
const GRAY_01: Color = Color::Rgb(194, 194, 194); // color_01
const GRAY_02: Color = Color::Rgb(171, 171, 171); // color_02
const GRAY_03: Color = Color::Rgb(149, 149, 149); // color_03
const GRAY_06: Color = Color::Rgb(36, 39, 43); // color_06
const GRAY_07: Color = Color::Rgb(24, 26, 29); // color_07
const GRAY_08: Color = Color::Rgb(17, 19, 21); // color_08
const GRAY_09: Color = Color::Rgb(12, 12, 12); // color_09 - 0c0c0c (background)
const BLUE: Color = Color::Rgb(88, 209, 235); // color_14, color_15 - #58d1eb (palette 6, 14)
const PINK: Color = Color::Rgb(244, 0, 95); // selection_background - #f4005f (palette 1, 5, 9, 13)
const PURPLE_05: Color = Color::Rgb(230, 216, 255); // color_10
const PURPLE_04: Color = Color::Rgb(206, 178, 255); // color_11
const PURPLE_03: Color = Color::Rgb(182, 140, 255); // color_12
const PURPLE_02: Color = Color::Rgb(157, 101, 255); // color_13 - Secondary - #9d65ff (palette 4, 12)

type Name = &'static str;

pub const SUNO_MONOKAI_REMASTERED_PALETTE: (Name, Palette) = (
    "Monokai Remastered",
    Palette {
        color_00: GRAY_00,
        color_01: GRAY_01,
        color_02: GRAY_02,
        color_03: GRAY_01,
        color_04: GRAY_03,
        color_05: GRAY_07,
        color_06: GRAY_06,
        color_07: GRAY_07,
        color_08: GRAY_08,
        color_09: GRAY_09,
        color_10: PURPLE_05,
        color_11: PURPLE_04,
        color_12: PURPLE_03,
        color_13: PURPLE_02,
        color_14: BLUE,
        color_15: BLUE,
        background: GRAY_09,
        foreground: GRAY_00,
        cursor_color: GRAY_06,
        cursor_text: GRAY_00,
        selection_background: PINK,
        selection_foreground: GRAY_09,
    },
);
