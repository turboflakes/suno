/// Signer specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("SecretError error: {0}")]
    SecretError(#[from] subxt_signer::SecretUriError),
    #[error("Keypair error: {0}")]
    KeypairError(#[from] subxt_signer::sr25519::Error),
    #[error("Decrypt error: {0}")]
    DecryptError(#[from] subxt_signer::polkadot_js_compat::Error),
    // #[error("SS58 error: {0}")]
    // FromSs58Error(#[from] subxt_utils_accountid32::FromSs58Error),
    #[error("Invalid address {0}")]
    InvalidAddress(String),
    #[error("Signer {0} path not specified")]
    SignerPathNotFound(String),
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
