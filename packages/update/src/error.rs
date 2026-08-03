#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Already up to date")]
    AlreadyUpToDate,
    #[error("Request error: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Unsupported architecture: {0}")]
    UnsupportedArchitecture(String),
    #[error("Unsupported OS: {0}")]
    UnsupportedOs(String),
    #[error("Asset not found: {0}")]
    AssetNotFound(String),
    #[error("No checksum found: {0}")]
    ChecksumNotFound(String),
    #[error("Invalid checksum format")]
    InvalidChecksumFormat,
    #[error("Invalid checksum")]
    InvalidChecksum,
    #[error("Binary not found")]
    BinaryNotFound,
    #[error("Unknown format: {0}")]
    UnknownFormat(String),
    #[error("Other error: {0}")]
    Other(String),
}
