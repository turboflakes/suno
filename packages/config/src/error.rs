/// Config specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Rpc error: {0}")]
    SubxtRpc(#[from] subxt_rpcs::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("Genesis error: {0}")]
    Genesis(String),
    #[error("At least one chain has to be enabled [Polkadot, Kusama, Paseo, Westend]")]
    ChainNotAvailable,
    #[error("Invalid proxy path")]
    InvalidProxyPath,
    #[error("Invalid proxy content")]
    InvalidProxyContent,
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
