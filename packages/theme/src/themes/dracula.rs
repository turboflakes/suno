use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the Ghostty "Dracula" theme:
// https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/ghostty/Dracula

const GRAY_00: Color = Color::Rgb(248, 248, 242); // color_00 - #f8f8f2 (foreground)
const GRAY_01: Color = Color::Rgb(225, 225, 221); // color_01
const GRAY_02: Color = Color::Rgb(202, 202, 200); // color_02
const GRAY_06: Color = Color::Rgb(68, 71, 90); // color_06 - #44475a (selection background)
const GRAY_07: Color = Color::Rgb(54, 56, 72); // color_07
const GRAY_08: Color = Color::Rgb(46, 48, 62); // color_08
const GRAY_09: Color = Color::Rgb(40, 42, 54); // color_09 - #282a36 (background)
const PURPLE_05: Color = Color::Rgb(242, 230, 255); // color_10
const PURPLE_04: Color = Color::Rgb(229, 205, 255); // color_11
const PURPLE_03: Color = Color::Rgb(214, 172, 255); // color_12 - #d6acff (palette 12)
const PURPLE_02: Color = Color::Rgb(189, 147, 249); // color_13 - Secondary - #bd93f9 (palette 4)
const CYAN: Color = Color::Rgb(139, 233, 253); // color_14, color_15 - #8be9fd (palette 6)
const YELLOW: Color = Color::Rgb(241, 250, 140); // color_04 - #f1fa8c (palette 3)
const PINK: Color = Color::Rgb(255, 121, 198); // selection_background - #ff79c6 (palette 5)

type Name = &'static str;

pub const SUNO_DRACULA_PALETTE: (Name, Palette) = (
    "Dracula",
    Palette {
        color_00: GRAY_00,
        color_01: GRAY_01,
        color_02: GRAY_02,
        color_03: GRAY_01,
        color_04: YELLOW,
        color_05: GRAY_07,
        color_06: GRAY_06,
        color_07: GRAY_07,
        color_08: GRAY_08,
        color_09: GRAY_09,
        color_10: PURPLE_05,
        color_11: PURPLE_04,
        color_12: PURPLE_03,
        color_13: PURPLE_02,
        color_14: CYAN,
        color_15: CYAN,
        background: GRAY_09,
        foreground: GRAY_00,
        cursor_color: GRAY_06,
        cursor_text: GRAY_00,
        selection_background: PINK,
        selection_foreground: GRAY_09,
    },
);
