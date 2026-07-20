/// Centralised error types for the NearField application.
///
/// Every fallible operation in the system returns `AppResult<T>`,
/// which is an alias for `Result<T, AppError>`. This keeps error
/// handling uniform across NFC, database, display, and config layers.
///
/// # Design
/// We use `thiserror` to derive `std::error::Error` with zero boilerplate.
/// The `#[from]` attribute auto-generates `From` impls for wrapped types
/// like `serialport::Error` and `rusqlite::Error`, enabling the `?` operator
/// throughout the codebase.

use thiserror::Error;

/// Alias for `Result<T, AppError>` used everywhere in this crate.
pub type AppResult<T> = Result<T, AppError>;

/// Every error that can occur in the NearField system.
#[derive(Error, Debug)]
pub enum AppError {
    /// Serial port could not be opened or encountered a low-level I/O error.
    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),

    /// PN532 chip returned unexpected data, invalid checksum, or a NAK.
    #[error("PN532 communication error: {0}")]
    Pn532Protocol(String),

    /// No NFC tag was detected in the RF field during this poll cycle.
    #[error("No tag detected")]
    NoTag,

    /// SQLite operation failed (connection, migration, query, etc.).
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    /// Display backend encountered an error (pipe broken, init failed, etc.).
    #[error("Display error: {0}")]
    Display(String),

    /// Configuration file is missing, malformed, or contains invalid values.
    #[error("Configuration error: {0}")]
    Config(String),

    /// General filesystem or I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Catch-all for other errors (JSON serialisation, signal handler setup, etc.).
    #[error("{0}")]
    Other(String),
}

impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::Other(msg)
    }
}

impl From<&str> for AppError {
    fn from(msg: &str) -> Self {
        AppError::Other(msg.to_string())
    }
}
