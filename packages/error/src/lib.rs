use suno_actions::Action;
use suno_config::SupportedRuntime;
use suno_signer::error::Error as SignerError;

/// Suno specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Backend error: {0}")]
    BackendError(#[from] subxt::error::BackendError),
    #[error("Events error: {0}")]
    EventsError(#[from] subxt::error::EventsError),
    #[error("Extrinsic error: {0}")]
    ExtrinsicError(#[from] subxt::error::ExtrinsicError),
    #[error("Transaction progress error: {0}")]
    TransactionProgressError(#[from] subxt::error::TransactionProgressError),
    #[error("Transaction status error: {0}")]
    TransactionStatusError(#[from] subxt::error::TransactionStatusError),
    #[error("Storage error: {0}")]
    StorageError(#[from] subxt::error::StorageError),
    #[error("Storage value error: {0}")]
    StorageValueError(#[from] subxt::ext::subxt_core::error::StorageValueError),
    #[error("Constant error: {0}")]
    ConstantError(#[from] subxt::ext::subxt_core::error::ConstantError),
    #[error("Send error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Action>),
    #[error("Signer error: {0}")]
    SignerError(#[from] SignerError),
    #[error("Genesis hash does not match the expected hash from the configured chain.")]
    GenesisError,
    #[error("Unsupported call: {0}")]
    UnsupportedCall(String),
    #[error("Clipboard error: {0}")]
    ClipboardError(#[from] arboard::Error),
    #[error("Unsupported runtime: {0}")]
    UnsupportedRuntime(SupportedRuntime),
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
