use suno_actions::Action;

/// Suno specific error messages
#[derive(thiserror::Error, Debug)]
pub enum TuiError {
    #[error("Signer error: {0}")]
    Signer(#[from] suno_signer::error::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Send error: {0}")]
    Send(#[from] Box<tokio::sync::mpsc::error::SendError<Action>>),
    #[error("Logger error: {0}")]
    TuiLogger(#[from] tui_logger::TuiLoggerError),
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
        Self::Other(error)
    }
}
