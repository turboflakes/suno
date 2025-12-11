/// Config specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Rpc error: {0}")]
    SubxtRpcError(#[from] subxt_rpcs::Error),
    #[error("Subxt error: {0}")]
    SubxtError(#[from] subxt::error::Error),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("YAML parsing error: {0}")]
    YamlError(#[from] serde_yaml::Error),
    #[error("Genesis error: {0}")]
    GenesisError(String),
    #[error("At least one chain has to be enabled [Polkadot, Kusama, Paseo, Westend]")]
    ChainNotAvailableError,
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
        Self::Other(error.into())
    }
}
