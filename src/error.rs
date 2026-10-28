//! Defines the custom error type for the application.
//!
//! `AppError` is an enum that consolidates all possible errors that can occur
//! during the data processing pipeline, from I/O and parsing to filtering logic.
//! This allows for centralized error handling and clear propagation.

use std::fmt;

// Using a custom error enum for more granular error handling.
#[derive(Debug)]
pub enum AppError {
    ParseError(String),
    InvalidFilter(String),
    RecordProcessingError(String),
    Io(std::io::Error),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            AppError::InvalidFilter(expr) => {
                write!(f, "Invalid filter expression: '{}'. Expected format: key=value", expr)
            }
            AppError::RecordProcessingError(msg) => write!(f, "Record processing error: {}", msg),
            AppError::Io(err) => write!(f, "I/O error: {}", err),
        }
    }
}

impl std::error::Error for AppError {}

impl From<std::io::Error> for AppError {
    fn from(err: std::io::Error) -> Self {
        AppError::Io(err)
    }
}

impl From<serde_json::Error> for AppError {
    fn from(err: serde_json::Error) -> Self {
        AppError::ParseError(err.to_string())
    }
}

impl From<csv_async::Error> for AppError {
    fn from(err: csv_async::Error) -> Self {
        AppError::ParseError(err.to_string())
    }
}
