use crate::error::Error;
use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn default_max_entries() -> usize {
    2000
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Logs {
    #[serde(default = "default_max_entries")]
    max_entries: usize,
    #[serde(default)]
    file_path: Option<String>,
}

impl Default for Logs {
    fn default() -> Self {
        Self {
            max_entries: default_max_entries(),
            file_path: None,
        }
    }
}

impl Logs {
    /// Constructs a `Logs` with the default filename path.
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let path = path.as_ref();
        let _ = path
            .file_name()
            .ok_or(Error::InvalidPath(path.display().to_string()))?;

        Ok(Logs {
            file_path: Some(path.to_string_lossy().into_owned()),
            max_entries: default_max_entries(),
        })
    }

    pub fn file_path(&self) -> Option<&str> {
        self.file_path.as_deref()
    }

    pub fn max_entries(&self) -> usize {
        self.max_entries
    }
}
