use crate::palette::Palette;
use ratatui::style::Color;

const BLUE_00: Color = Color::Rgb(248, 250, 252); // color_00
const BLUE_01: Color = Color::Rgb(234, 240, 246); // color_01
const BLUE_02: Color = Color::Rgb(219, 230, 240); // color_02
const BLUE_03: Color = Color::Rgb(205, 219, 234); // color_03
const BLUE_04: Color = Color::Rgb(191, 209, 227); // color_04
const BLUE_05: Color = Color::Rgb(46, 77, 107); // color_05
const BLUE_06: Color = Color::Rgb(38, 64, 89); // color_06
const BLUE_07: Color = Color::Rgb(31, 51, 71); // color_07
const BLUE_08: Color = Color::Rgb(23, 38, 54); // color_08
const BLUE_09: Color = Color::Rgb(15, 25, 35); // color_09
const YELLOW_05: Color = Color::Rgb(255, 254, 245); // color_10
const YELLOW_04: Color = Color::Rgb(255, 252, 229); // color_11
const YELLOW_03: Color = Color::Rgb(255, 249, 204); // color_12
const YELLOW_02: Color = Color::Rgb(255, 246, 179); // color_13
const YELLOW_01: Color = Color::Rgb(255, 240, 128); // color_14
const YELLOW_00: Color = Color::Rgb(255, 237, 102); // color_15

type Name = &'static str;

pub const SUNO_NIGHTRUN_PALETTE: (Name, Palette) = (
    "Nightrun",
    Palette {
        color_00: BLUE_00,
        color_01: BLUE_01,
        color_02: BLUE_02,
        color_03: BLUE_03,
        color_04: BLUE_04,
        color_05: BLUE_05,
        color_06: BLUE_06,
        color_07: BLUE_07,
        color_08: BLUE_08,
        color_09: BLUE_09,
        color_10: YELLOW_05,
        color_11: YELLOW_04,
        color_12: YELLOW_03,
        color_13: YELLOW_02,
        color_14: YELLOW_01,
        color_15: YELLOW_00,
        background: BLUE_09,
        foreground: BLUE_01,
        cursor_color: BLUE_08,
        cursor_text: BLUE_01,
        selection_background: BLUE_01,
        selection_foreground: BLUE_09,
    },
);
