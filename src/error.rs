use thiserror::Error;

/// Defines the custom error types for the application.
#[derive(Error, Debug)]
pub enum AppError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON serialization/deserialization error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("CSV processing error: {0}")]
    Csv(#[from] csv_async::Error),

    #[error("Failed to parse record: {0}")]
    ParseError(String),

    #[error("Invalid filter expression: {0}")]
    InvalidFilter(String),
}
