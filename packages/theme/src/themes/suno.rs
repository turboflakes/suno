use crate::palette::Palette;
use ratatui::style::Color;
//
// SUNO DUOTONE 16 COLORS
//
// Grays
// | Hex       | RGB             | Notes         |
// | --------- | --------------- | ------------- |
// | `#F8F7F7` | (248, 247, 247) | Lightest gray |
// | `#DCDBDB` | (220, 219, 219) |               |
// | `#C1BEBF` | (193, 190, 191) |               |
// | `#A5A1A3` | (165, 161, 163) |               |
// | `#8A8587` | (138, 133, 135) |               |
// | `#6D696B` | (109, 105, 107) |               |
// | `#514E4F` | (81, 78, 79)    |               |
// | `#343233` | (52, 50, 51)    | **Primary**   |
// | `#1F1E1F` | (31, 30, 31)    |               |
// | `#0A0A0A` | (10, 10, 10)    |               |
// | --------- | --------------- | ------------- |
//
// Yellows
// | Hex       | RGB             | Notes          |
// | --------- | --------------- | -------------- |
// | `#FEF5C2` | (254, 245, 194) | Light yellow   |
// | `#FEEB8B` | (254, 235, 139) |                |
// | `#FDE253` | (253, 226, 83)  |                |
// | `#FDD91E` | (253, 217, 30)  | **Secondary**  |
// | `#DEBB02` | (222, 187, 2)   |                |
// | `#A78C01` | (167, 140, 1)   | Darkest yellow |
// | --------- | --------------- | -------------- |

const GRAY_09: Color = Color::Rgb(248, 247, 247); // color_00
const GRAY_08: Color = Color::Rgb(220, 219, 219); // color_01
const GRAY_07: Color = Color::Rgb(193, 190, 191); // color_02
const GRAY_06: Color = Color::Rgb(165, 161, 163); // color_03
const GRAY_05: Color = Color::Rgb(138, 133, 135); // color_04
const GRAY_04: Color = Color::Rgb(109, 105, 107); // color_05
const GRAY_03: Color = Color::Rgb(81, 78, 79); // color_06
const GRAY_02: Color = Color::Rgb(52, 50, 51); // color_07 - Primary
const GRAY_01: Color = Color::Rgb(31, 30, 31); // color_08
const GRAY_00: Color = Color::Rgb(10, 10, 10); // color_09
const YELLOW_05: Color = Color::Rgb(254, 235, 139); // color_10
const YELLOW_04: Color = Color::Rgb(254, 235, 139); // color_11
const YELLOW_03: Color = Color::Rgb(253, 226, 83); // color_12
const YELLOW_02: Color = Color::Rgb(253, 217, 30); // color_13 - Secondary
const YELLOW_01: Color = Color::Rgb(222, 187, 2); // color_14
const YELLOW_00: Color = Color::Rgb(167, 140, 1); // color_15

type Name = &'static str;

pub const SUNO_DARK_PALETTE: (Name, Palette) = (
    "Suno Dark",
    Palette {
        color_00: GRAY_09,
        color_01: GRAY_08,
        color_02: GRAY_07,
        color_03: GRAY_06,
        color_04: GRAY_05,
        color_05: GRAY_04,
        color_06: GRAY_03,
        color_07: GRAY_02,
        color_08: GRAY_01,
        color_09: GRAY_00,
        color_10: YELLOW_05,
        color_11: YELLOW_04,
        color_12: YELLOW_03,
        color_13: YELLOW_02,
        color_14: YELLOW_01,
        color_15: YELLOW_00,
        background: GRAY_00,
        foreground: GRAY_08,
        cursor_color: GRAY_01,
        cursor_text: GRAY_08,
        selection_background: GRAY_09,
        selection_foreground: GRAY_00,
    },
);

pub const SUNO_LIGHT_PALETTE: (Name, Palette) = (
    "Suno Light",
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
        color_10: YELLOW_00,
        color_11: YELLOW_01,
        color_12: YELLOW_02,
        color_13: YELLOW_01,
        color_14: YELLOW_04,
        color_15: YELLOW_05,
        background: GRAY_09,
        foreground: GRAY_01,
        cursor_color: GRAY_08,
        cursor_text: GRAY_01,
        selection_background: GRAY_00,
        selection_foreground: GRAY_09,
    },
);
