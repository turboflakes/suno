use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the Ghostty "Borland" theme:
// https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/ghostty/Borland

const COLOR_00: Color = Color::Rgb(238, 238, 238); // eeeeee
const COLOR_01: Color = Color::Rgb(212, 212, 230);
const COLOR_02: Color = Color::Rgb(185, 185, 222);
const COLOR_03: Color = Color::Rgb(159, 159, 213);
const COLOR_04: Color = Color::Rgb(132, 132, 205);
const COLOR_05: Color = Color::Rgb(106, 106, 197);
const COLOR_06: Color = Color::Rgb(79, 79, 189);
const COLOR_07: Color = Color::Rgb(53, 53, 180);
const COLOR_08: Color = Color::Rgb(26, 26, 172);
const COLOR_09: Color = Color::Rgb(0, 0, 164); // 0000a4 (background)
const COLOR_10: Color = Color::Rgb(255, 255, 214);
const COLOR_11: Color = Color::Rgb(243, 234, 171);
const COLOR_12: Color = Color::Rgb(231, 213, 128);
const COLOR_13: Color = Color::Rgb(255, 115, 253); // Secondary - ff73fd (headers)
const COLOR_14: Color = Color::Rgb(208, 171, 43);
const COLOR_15: Color = Color::Rgb(196, 150, 0);

type Name = &'static str;

pub const SUNO_BORLAND_PALETTE: (Name, Palette) = (
    "Borland",
    Palette {
        color_00: COLOR_00,
        color_01: COLOR_01,
        color_02: COLOR_02,
        color_03: COLOR_03,
        color_04: COLOR_04,
        color_05: COLOR_05,
        color_06: COLOR_06,
        color_07: COLOR_07,
        color_08: COLOR_08,
        color_09: COLOR_09,
        color_10: COLOR_10,
        color_11: COLOR_11,
        color_12: COLOR_12,
        color_13: COLOR_13,
        color_14: COLOR_14,
        color_15: COLOR_15,
        background: COLOR_09,
        foreground: COLOR_01,
        cursor_color: COLOR_08,
        cursor_text: COLOR_01,
        selection_background: COLOR_01,
        selection_foreground: COLOR_09,
    },
);
