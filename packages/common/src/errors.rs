use super::actions::Action;
use subxt::error::{DispatchError, MetadataError, RpcError, TransactionError};

use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tui_logger::TuiLoggerError;

/// Snops specific error messages
#[derive(Error, Debug)]
pub enum SnopsError {
    #[error("Subxt error: {0}")]
    SubxtError(#[from] subxt::Error),
    #[error("SubxtCore error: {0}")]
    SubxtCoreError(#[from] subxt::ext::subxt_core::Error),
    #[error("Metadata error: {0}")]
    MetadataError(#[from] MetadataError),
    #[error("Dispatch error: {0}")]
    DispatchError(#[from] DispatchError),
    #[error("Rpc error: {0}")]
    RpcError(#[from] RpcError),
    #[error("IO error: {0}")]
    IoError(#[from] tokio::io::Error),
    #[error("Send error: {0}")]
    SendError(#[from] SendError<Action>),
    #[error("Logger error: {0}")]
    TuiLoggerError(#[from] TuiLoggerError),
    #[error("Tx error: {0}")]
    TransactionError(#[from] TransactionError),
    #[error("SecretError error: {0}")]
    SecretError(#[from] subxt_signer::SecretUriError),
    #[error("Keypair error: {0}")]
    KeypairError(#[from] subxt_signer::sr25519::Error),
    #[error("Other error: {0}")]
    Other(String),
}

/// Convert &str to SnopsError
impl From<&str> for SnopsError {
    fn from(error: &str) -> Self {
        Self::Other(error.into())
    }
}

/// Convert String to SnopsError
impl From<String> for SnopsError {
    fn from(error: String) -> Self {
        Self::Other(error.into())
    }
}
