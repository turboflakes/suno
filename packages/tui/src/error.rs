use suno_actions::Action;

/// Suno specific error messages
#[derive(thiserror::Error, Debug)]
pub enum TuiError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Subxt error: {0}")]
    SubxtError(#[from] subxt::Error),
    #[error("SubxtCore error: {0}")]
    SubxtCoreError(#[from] subxt::ext::subxt_core::Error),
    #[error("Metadata error: {0}")]
    MetadataError(#[from] subxt::error::MetadataError),
    #[error("Dispatch error: {0}")]
    DispatchError(#[from] subxt::error::DispatchError),
    #[error("Rpc error: {0}")]
    RpcError(#[from] subxt::error::RpcError),
    #[error("Send error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Action>),
    #[error("Logger error: {0}")]
    TuiLoggerError(#[from] tui_logger::TuiLoggerError),
    #[error("Tx error: {0}")]
    TransactionError(#[from] subxt::error::TransactionError),
    #[error("Genesis hash does not match the expected hash from the configured chain.")]
    GenesisError,
    #[error("Other error: {0}")]
    Other(String),
}

/// Convert &str to TuiError
impl From<&str> for TuiError {
    fn from(error: &str) -> Self {
        Self::Other(error.into())
    }
}

/// Convert String to TuiError
impl From<String> for TuiError {
    fn from(error: String) -> Self {
        Self::Other(error.into())
    }
}
