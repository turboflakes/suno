/// Signer specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Nokhwa error: {0}")]
    NokhwaError(#[from] nokhwa::error::NokhwaError),
    #[error("Genesis hash not available")]
    GenesisHashNotAvailable,
    #[error("Other error: {0}")]
    Other(String),
}
