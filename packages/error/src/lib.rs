use suno_actions::Action;
use suno_config::SupportedRuntime;
use suno_primitives::{session::KeysError, tx::Error as PayloadError};

/// Suno specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OnlineClient error: {0}")]
    OnlineClient(#[from] Box<subxt::error::OnlineClientAtBlockError>),
    #[error("Backend error: {0}")]
    Backend(#[from] Box<subxt::error::BackendError>),
    #[error("Block error: {0}")]
    Block(#[from] Box<subxt::error::BlockError>),
    #[error("Events error: {0}")]
    Events(#[from] Box<subxt::error::EventsError>),
    #[error("Extrinsic error: {0}")]
    Extrinsic(#[from] Box<subxt::error::ExtrinsicError>),
    #[error("Transaction progress error: {0}")]
    TransactionProgress(#[from] Box<subxt::error::TransactionProgressError>),
    #[error("Transaction status error: {0}")]
    TransactionStatus(#[from] Box<subxt::error::TransactionStatusError>),
    #[error("Storage error: {0}")]
    Storage(#[from] Box<subxt::error::StorageError>),
    #[error("Storage value error: {0}")]
    StorageValue(#[from] Box<subxt::error::StorageValueError>),
    #[error("Constant error: {0}")]
    Constant(#[from] Box<subxt::error::ConstantError>),
    #[error("Send error: {0}")]
    Send(#[from] Box<tokio::sync::mpsc::error::SendError<Action>>),
    #[error("Signer error: {0}")]
    Signer(#[from] suno_signer::Error),
    #[error("Payload error: {0}")]
    Payload(#[from] Box<PayloadError>),
    #[error("Keys error: {0}")]
    Keys(#[from] Box<KeysError>),
    #[error("Genesis hash does not match the expected hash from the configured chain.")]
    Genesis,
    #[error("Unsupported call: {0}")]
    UnsupportedCall(String),
    #[error("Clipboard error: {0}")]
    Clipboard(#[from] arboard::Error),
    #[error("Unsupported runtime: {0}")]
    UnsupportedRuntime(SupportedRuntime),
    #[error("Config error: {0}")]
    Config(#[from] Box<suno_config::Error>),
    #[error("Invalid signature: {0}")]
    InvalidSignature(#[from] subxt::ext::codec::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Logger error: {0}")]
    Tracing(#[from] suno_tracing::Error),
    #[error("Logger error: {0}")]
    TuiLogger(#[from] tui_logger::TuiLoggerError),
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
