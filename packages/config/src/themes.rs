use crate::error::Error;
use log::warn;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use suno_theme::{Palette, Theme, SUNO_DARK_PALETTE, SUNO_LIGHT_PALETTE};

/// Provides default value for the themes directory
fn default_themes_path() -> String {
    "themes".to_string()
}

/// Provides default value for the active theme
fn default_active_theme() -> String {
    "Suno Dark".to_string()
}

/// Provides default value for the active theme
fn default_themes() -> HashMap<String, Theme> {
    // Register SUNO built-ins Themes
    let mut themes = HashMap::new();
    themes.insert("Suno Dark".into(), Theme::from_palette(&SUNO_DARK_PALETTE));
    themes.insert(
        "Suno Light".into(),
        Theme::from_palette(&SUNO_LIGHT_PALETTE),
    );
    themes
}

#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Themes {
    #[serde(default = "default_themes_path")]
    pub themes_path: String,
    #[serde(default = "default_active_theme")]
    pub active_theme: String,
    #[serde(skip)]
    #[serde(default = "default_themes")]
    themes: HashMap<String, Theme>,
}

impl Themes {
    pub fn theme(&self) -> &Theme {
        self.themes
            .get(&self.active_theme)
            .expect("no themes loaded")
    }

    pub fn set_themes(&mut self, themes: HashMap<String, Theme>) {
        self.themes = themes;
    }

    pub fn load<P: AsRef<Path>>(path_dir: P) -> Result<HashMap<String, Theme>, Error> {
        let path_dir = path_dir.as_ref();

        if !path_dir.exists() {
            warn!("Themes directory does not exist: {}", path_dir.display());
            return Err(Error::InvalidPath(path_dir.display().to_string()));
        }

        // Register SUNO built-ins Themes
        let mut themes = HashMap::new();
        themes.insert("Suno Dark".into(), Theme::from_palette(&SUNO_DARK_PALETTE));
        themes.insert(
            "Suno Light".into(),
            Theme::from_palette(&SUNO_LIGHT_PALETTE),
        );

        // Scan {path_dir}/*.toml
        if let Ok(entries) = std::fs::read_dir(&path_dir) {
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
                    .to_string();

                let palette = Palette::from_file(path)?;
                themes.insert(name, Theme::from_palette(&palette));
            }
        }

        Ok(themes)
    }
}
