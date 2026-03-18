/// Config specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Invalid content: {0}")]
    InvalidContent(String),
    #[error("Invalid color: {0}")]
    InvalidColor(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parsing error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Other error: {0}")]
    Other(String),
}

/// Convert &str to Error
impl From<&str> for Error {
    fn from(error: &str) -> Self {
        Self::Other(error.into())
    }
}

/// Convert String to Error
impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Other(error)
    }
}
