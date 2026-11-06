use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the Ghostty "TokyoNight" theme:
// https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/ghostty/TokyoNight

const GRAY_00: Color = Color::Rgb(192, 202, 245); // color_00, color_01, color_03 - #c0caf5 (foreground, palette 15)
const GRAY_02: Color = Color::Rgb(169, 177, 214); // color_02 - #a9b1d6 (palette 7)
const PURPLE: Color = Color::Rgb(187, 154, 247); // color_04 - #bb9af7 (palette 5, 13)
const GRAY_06: Color = Color::Rgb(41, 46, 66); // color_06
const GRAY_07: Color = Color::Rgb(36, 40, 59); // color_07
const GRAY_08: Color = Color::Rgb(31, 33, 47); // color_08
const GRAY_09: Color = Color::Rgb(26, 27, 38); // color_09 - #1a1b26 (background)
const BLUE_05: Color = Color::Rgb(222, 232, 253); // color_10
const BLUE_04: Color = Color::Rgb(188, 208, 251); // color_11
const BLUE_03: Color = Color::Rgb(155, 185, 249); // color_12
const BLUE_02: Color = Color::Rgb(122, 162, 247); // color_13 - Secondary - #7aa2f7 (palette 4, 12)
const CYAN: Color = Color::Rgb(125, 207, 255); // color_14, color_15 - #7dcfff (palette 6, 14)
const SELECTION: Color = Color::Rgb(51, 70, 124); // cursor_color, selection_background - #33467c

type Name = &'static str;

pub const SUNO_TOKYO_NIGHT_PALETTE: (Name, Palette) = (
    "Tokyo Night",
    Palette {
        color_00: GRAY_00,
        color_01: GRAY_00,
        color_02: GRAY_02,
        color_03: GRAY_00,
        color_04: PURPLE,
        color_05: GRAY_07,
        color_06: GRAY_06,
        color_07: GRAY_07,
        color_08: GRAY_08,
        color_09: GRAY_09,
        color_10: BLUE_05,
        color_11: BLUE_04,
        color_12: BLUE_03,
        color_13: BLUE_02,
        color_14: CYAN,
        color_15: CYAN,
        background: GRAY_09,
        foreground: GRAY_00,
        cursor_color: SELECTION,
        cursor_text: GRAY_00,
        selection_background: SELECTION,
        selection_foreground: GRAY_00,
    },
);
