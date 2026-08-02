/// Config specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Invalid path: {0}")]
    InvalidPath(String),
    #[error("Invalid content: {0}")]
    InvalidContent(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("At least one chain has to be configured [Polkadot, Kusama, Paseo, Westend]")]
    ChainNotAvailable,
    #[error("Theme parsing error: {0}")]
    Theme(#[from] suno_theme::Error),
    #[error("Invalid theme: {0}")]
    InvalidTheme(String),
    #[error("Invalid command name. '{0}' is a reserved word")]
    InvalidCommand(String),
    #[error("Local execution error: {0}")]
    LocalExecution(String),
    #[error("Invalid address {0}")]
    InvalidAddress(String),
    #[error("Signer not defined")]
    SignerNotDefined,
    #[error("Remote execution error: {0}")]
    RemoteExecution(String),
    #[error("Invalid version: {0}")]
    InvalidVersion(semver::Error),
    #[error("Invalid version: {0}")]
    RequestError(reqwest::Error),
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

/// Convert reqwest::Error to Error
impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Self::RequestError(error)
    }
}
