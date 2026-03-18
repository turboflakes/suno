use crate::error::Error;
use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::str::FromStr;
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

pub struct Palette {
    pub color_00: Color,
    pub color_01: Color,
    pub color_02: Color,
    pub color_03: Color,
    pub color_04: Color,
    pub color_05: Color,
    pub color_06: Color,
    pub color_07: Color,
    pub color_08: Color,
    pub color_09: Color,
    pub color_10: Color,
    pub color_11: Color,
    pub color_12: Color,
    pub color_13: Color,
    pub color_14: Color,
    pub color_15: Color,
    pub background: Color,
    pub foreground: Color,
    pub cursor_color: Color,
    pub cursor_text: Color,
    pub selection_background: Color,
    pub selection_foreground: Color,
}

impl Default for Palette {
    fn default() -> Self {
        SUNO_DARK_PALETTE.1
    }
}

#[derive(Deserialize)]
pub struct PaletteRaw {
    pub color_00: String,
    pub color_01: String,
    pub color_02: String,
    pub color_03: String,
    pub color_04: String,
    pub color_05: String,
    pub color_06: String,
    pub color_07: String,
    pub color_08: String,
    pub color_09: String,
    pub color_10: String,
    pub color_11: String,
    pub color_12: String,
    pub color_13: String,
    pub color_14: String,
    pub color_15: String,
    pub background: String,
    pub foreground: String,
    pub cursor_color: String,
    pub cursor_text: String,
    pub selection_background: String,
    pub selection_foreground: String,
}

fn parse_color(s: &str) -> Result<Color, Error> {
    Color::from_str(s).map_err(|_| Error::InvalidColor(s.to_string()))
}

impl TryFrom<PaletteRaw> for Palette {
    type Error = Error;
    fn try_from(r: PaletteRaw) -> Result<Self, Error> {
        Ok(Palette {
            color_00: parse_color(&r.color_00)?,
            color_01: parse_color(&r.color_01)?,
            color_02: parse_color(&r.color_02)?,
            color_03: parse_color(&r.color_03)?,
            color_04: parse_color(&r.color_04)?,
            color_05: parse_color(&r.color_05)?,
            color_06: parse_color(&r.color_06)?,
            color_07: parse_color(&r.color_07)?,
            color_08: parse_color(&r.color_08)?,
            color_09: parse_color(&r.color_09)?,
            color_10: parse_color(&r.color_10)?,
            color_11: parse_color(&r.color_11)?,
            color_12: parse_color(&r.color_12)?,
            color_13: parse_color(&r.color_13)?,
            color_14: parse_color(&r.color_14)?,
            color_15: parse_color(&r.color_15)?,
            background: parse_color(&r.background)?,
            foreground: parse_color(&r.foreground)?,
            cursor_color: parse_color(&r.cursor_color)?,
            cursor_text: parse_color(&r.cursor_text)?,
            selection_background: parse_color(&r.selection_background)?,
            selection_foreground: parse_color(&r.selection_foreground)?,
        })
    }
}

impl Palette {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(Error::InvalidPath(path.display().to_string()));
        }

        let content = fs::read_to_string(path)?;
        if content.is_empty() {
            return Err(Error::InvalidContent(path.display().to_string()));
        }

        let raw: PaletteRaw = toml::from_str(&content)?;
        let palette = Palette::try_from(raw)?;

        Ok(palette)
    }
}
