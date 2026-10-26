use thiserror::Error;

/// Defines the custom error types for the application, providing a centralized
/// and structured way to handle various failure scenarios.
#[derive(Error, Debug)]
pub enum AppError {
    /// Represents an I/O error that may occur during file operations.
    /// This variant wraps the standard `std::io::Error` for seamless integration.
    #[error("I/O error")]
    Io(#[from] std::io::Error),
}
