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
    #[error("No secret key available")]
    NoKey,
    #[error("SecretError error: {0}")]
    SecretError(#[from] subxt_signer::SecretUriError),
    #[error("Keypair error: {0}")]
    KeypairError(#[from] subxt_signer::sr25519::Error),
    #[error("At least one chain has to be configured [Polkadot, Kusama, Paseo, Westend]")]
    ChainNotAvailable,
    #[error("Unsupported chain: {0}")]
    UnsupportedChain(String),
    #[error("Unsupported provider: {0}")]
    UnsupportedProvider(String),
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
    #[error("Reqwest error: {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Extrinsic error: {0}")]
    Extrinsic(#[from] Box<subxt::error::ExtrinsicError>),
    #[error("Runtime API error: {0}")]
    RuntimeApi(#[from] Box<subxt::error::RuntimeApiError>),
    #[error("Codec error: {0}")]
    Codec(#[from] subxt::ext::codec::Error),
    #[error("Genesis hash not available")]
    GenesisHashNotAvailable,
    #[error("Metadata V14 not available")]
    MetadataV14NotAvailable,
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

/// Extension trait to box errors in Results, keeping the `Error` enum small.
pub trait ResultExt<T, E> {
    fn boxed(self) -> Result<T, Box<E>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn boxed(self) -> Result<T, Box<E>> {
        self.map_err(Box::new)
    }
}
