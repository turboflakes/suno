use crate::actions::Action;
use subxt::error::{DispatchError, MetadataError, RpcError};
use thiserror::Error;
use tokio::sync::mpsc::error::SendError;
use tui_logger::TuiLoggerError;

/// Claimit specific error messages
#[derive(Error, Debug)]
pub enum TuiError {
    #[error("Subxt error: {0}")]
    SubxtError(#[from] subxt::Error),
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
    #[error("Other error: {0}")]
    Other(String),
}

/// Convert &str to TuiError
impl From<&str> for TuiError {
    fn from(error: &str) -> Self {
        TuiError::Other(error.into())
    }
}
