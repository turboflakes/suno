use suno_actions::Action;
use suno_signer::error::Error as SignerError;

/// Suno specific error messages
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Signer error: {0}")]
    SignerError(#[from] SignerError),
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
    #[error("Tx error: {0}")]
    TransactionError(#[from] subxt::error::TransactionError),
    #[error("SendError error: {0}")]
    SendError(#[from] tokio::sync::mpsc::error::SendError<Action>),
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
