use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the colors used by https://kusama.network

const WHITE: Color = Color::Rgb(255, 255, 255); // color_00, color_01, color_03, foreground, cursor_text - #ffffff
const GRAY_02: Color = Color::Rgb(204, 204, 204); // color_02 - #cccccc
const GRAY_04: Color = Color::Rgb(178, 178, 178); // color_04 - #b2b2b2
const PURPLE_06: Color = Color::Rgb(17, 0, 17); // color_06, cursor_color - #110011
const PURPLE_08: Color = Color::Rgb(10, 0, 8); // color_05, color_07, color_08 - #0a0008
const BLACK: Color = Color::Rgb(0, 0, 0); // color_09, background, selection_foreground - #000000
const PINK: Color = Color::Rgb(255, 0, 102); // color_10, color_11, color_12, color_13, color_14, color_15, selection_background - #ff0066

type Name = &'static str;

pub const SUNO_KUSAMA_PALETTE: (Name, Palette) = (
    "Kusama",
    Palette {
        color_00: WHITE,
        color_01: WHITE,
        color_02: GRAY_02,
        color_03: WHITE,
        color_04: GRAY_04,
        color_05: PURPLE_08,
        color_06: PURPLE_06,
        color_07: PURPLE_08,
        color_08: PURPLE_08,
        color_09: BLACK,
        color_10: PINK,
        color_11: PINK,
        color_12: PINK,
        color_13: PINK,
        color_14: PINK,
        color_15: PINK,
        background: BLACK,
        foreground: WHITE,
        cursor_color: PURPLE_06,
        cursor_text: WHITE,
        selection_background: PINK,
        selection_foreground: BLACK,
    },
);
