use crate::error::Error;
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use suno_theme::{Palette, Theme, SUNO_DARK_PALETTE, SUNO_LIGHT_PALETTE};

type Name = String;
type ThemesMap = HashMap<Name, Theme>;

/// Provides default value for the themes directory
fn default_themes_path() -> String {
    "./themes".to_string()
}

/// Provides default value for the active theme
pub fn default_active_theme() -> String {
    SUNO_DARK_PALETTE.0.into()
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Themes {
    #[serde(default = "default_themes_path")]
    pub path: String,
    #[serde(default = "default_active_theme")]
    pub active: String,
    #[serde(skip)]
    themes: ThemesMap,
}

impl Themes {
    pub fn theme(&self) -> &Theme {
        self.themes
            .get(&self.active)
            .or_else(|| self.themes.get(&default_active_theme()))
            .expect("No theme loaded")
    }

    pub fn set_themes(&mut self, themes: ThemesMap) {
        self.themes = themes;
    }

    pub fn validate(&self) -> Result<(), Error> {
        if !self.themes.contains_key(&self.active) {
            return Err(Error::InvalidTheme(self.active.clone()));
        }
        info!("Theme: {}", self.active);
        Ok(())
    }

    pub fn load<P: AsRef<Path>>(path_dir: P) -> Result<ThemesMap, Error> {
        let path_dir = path_dir.as_ref();

        // Register SUNO builtins Themes
        let mut themes = HashMap::new();
        themes.insert(
            SUNO_DARK_PALETTE.0.into(),
            Theme::from_palette(&SUNO_DARK_PALETTE.1),
        );
        themes.insert(
            SUNO_LIGHT_PALETTE.0.into(),
            Theme::from_palette(&SUNO_LIGHT_PALETTE.1),
        );

        if !path_dir.is_dir() {
            warn!("Themes directory does not exist: {}", path_dir.display());
            return Ok(themes);
        }

        // Scan {path_dir}/*.toml
        if let Ok(entries) = std::fs::read_dir(path_dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                // Skip files that are not .toml
                if path.extension().and_then(|e| e.to_str()) != Some("toml") {
                    continue;
                }

                let name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("unknown")
                    .trim()
                    .to_string();

                let palette = Palette::from_file(path)?;
                themes.insert(name, Theme::from_palette(&palette));
            }
        }

        Ok(themes)
    }
}
