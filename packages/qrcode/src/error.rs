/// Signer specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Extrinsic error: {0}")]
    Extrinsic(#[from] Box<subxt::error::ExtrinsicError>),
    #[error("Nokhwa error: {0}")]
    NokhwaError(#[from] nokhwa::error::NokhwaError),
    #[error("Genesis hash not available")]
    GenesisHashNotAvailable,
    #[error("Other error: {0}")]
    Other(String),
}

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
