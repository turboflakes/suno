use crate::error::Error;
use crate::themes::suno::SUNO_DARK_PALETTE;
use ratatui::style::Color;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::str::FromStr;

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
