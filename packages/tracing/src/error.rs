#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Tracing error: {0}")]
    TracingError(String),
    #[error("Invalid filename")]
    InvalidFilename,
}
