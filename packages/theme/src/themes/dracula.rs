use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the Ghostty "Dracula" theme:
// https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/ghostty/Dracula

const GRAY_00: Color = Color::Rgb(248, 248, 242); // color_00 - f8f8f2 (foreground)
const GRAY_01: Color = Color::Rgb(225, 225, 221); // color_01
const GRAY_02: Color = Color::Rgb(202, 202, 200); // color_02
const GRAY_03: Color = Color::Rgb(179, 179, 179); // color_03
const GRAY_04: Color = Color::Rgb(156, 156, 158); // color_04
const GRAY_05: Color = Color::Rgb(132, 134, 138); // color_05
const GRAY_06: Color = Color::Rgb(109, 111, 117); // color_06
const GRAY_07: Color = Color::Rgb(86, 88, 96); // color_07
const GRAY_08: Color = Color::Rgb(63, 65, 75); // color_08
const GRAY_09: Color = Color::Rgb(40, 42, 54); // color_09 - 282a36 (background)
const PURPLE_05: Color = Color::Rgb(222, 197, 255); // color_10
const PURPLE_04: Color = Color::Rgb(197, 169, 238); // color_11
const PURPLE_03: Color = Color::Rgb(172, 141, 220); // color_12
const PURPLE_02: Color = Color::Rgb(148, 114, 203); // color_13 - Secondary
const PURPLE_01: Color = Color::Rgb(123, 86, 185); // color_14
const PURPLE_00: Color = Color::Rgb(98, 58, 168); // color_15

type Name = &'static str;

pub const SUNO_DRACULA_PALETTE: (Name, Palette) = (
    "Dracula",
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
        color_10: PURPLE_05,
        color_11: PURPLE_04,
        color_12: PURPLE_03,
        color_13: PURPLE_02,
        color_14: PURPLE_01,
        color_15: PURPLE_00,
        background: GRAY_09,
        foreground: GRAY_01,
        cursor_color: GRAY_08,
        cursor_text: GRAY_01,
        selection_background: GRAY_01,
        selection_foreground: GRAY_09,
    },
);
