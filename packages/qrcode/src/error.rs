/// Signer specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Extrinsic error: {0}")]
    Extrinsic(#[from] Box<subxt::error::ExtrinsicError>),
    #[error("Nokhwa error: {0}")]
    NokhwaError(#[from] nokhwa::error::NokhwaError),
    // #[error("IO error: {0}")]
    // IoError(#[from] std::io::Error),
    // #[error("SecretError error: {0}")]
    // SecretError(#[from] subxt_signer::SecretUriError),
    // #[error("Keypair error: {0}")]
    // KeypairError(#[from] subxt_signer::sr25519::Error),
    // #[error("Decrypt error: {0}")]
    // DecryptError(#[from] subxt_signer::polkadot_js_compat::Error),
    // #[error("Invalid address {0}")]
    // InvalidAddress(String),
    #[error("Genesis hash not available")]
    GenesisHashNotAvailable,
    // #[error("Path not found")]
    // PathNotFound,
    #[error("Other error: {0}")]
    Other(String),
}

// /// Convert &str to Error
// impl From<&str> for Error {
//     fn from(error: &str) -> Self {
//         Self::Other(error.into())
//     }
// }

// /// Convert String to Error
// impl From<String> for Error {
//     fn from(error: String) -> Self {
//         Self::Other(error)
//     }
// }

/// Extension trait to box errors in Results
pub trait ResultExt<T, E> {
    /// Box the error variant for smaller Result sizes
    fn boxed(self) -> Result<T, Box<E>>;
}

impl<T, E> ResultExt<T, E> for Result<T, E> {
    fn boxed(self) -> Result<T, Box<E>> {
        self.map_err(Box::new)
    }
}
