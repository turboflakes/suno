use crate::palette::Palette;
use ratatui::style::Color;

// Adapted from the Ghostty "Homebrew" theme:
// https://github.com/mbadolato/iTerm2-Color-Schemes/blob/master/ghostty/Homebrew

const GRAY_00: Color = Color::Rgb(229, 229, 229); // color_00
const GRAY_07: Color = Color::Rgb(42, 42, 42); // color_07
const GRAY_08: Color = Color::Rgb(21, 21, 21); // color_08
const GRAY_09: Color = Color::Rgb(0, 0, 0); // color_09 - background
const WHITE: Color = Color::Rgb(255, 255, 255);
const BLACK: Color = Color::Rgb(0, 0, 0);
const GREEN_01: Color = Color::Rgb(0, 217, 0);
const GREEN_02: Color = Color::Rgb(0, 166, 0);
const GREEN_03: Color = Color::Rgb(8, 57, 5);

type Name = &'static str;

pub const SUNO_HOMEBREW_PALETTE: (Name, Palette) = (
    "Homebrew",
    Palette {
        color_00: GRAY_00,
        color_01: GRAY_00,
        color_02: GRAY_00,
        color_03: GRAY_00,
        color_04: GRAY_07,
        color_05: GRAY_08,
        color_06: GRAY_09,
        color_07: GRAY_07,
        color_08: GRAY_08,
        color_09: GRAY_09,
        color_10: GREEN_01,
        color_11: GREEN_01,
        color_12: GREEN_02,
        color_13: GREEN_02,
        color_14: GREEN_03,
        color_15: GREEN_03,
        background: BLACK,
        foreground: GREEN_01,
        cursor_color: BLACK,
        cursor_text: GREEN_01,
        selection_background: GREEN_03,
        selection_foreground: WHITE,
    },
);
