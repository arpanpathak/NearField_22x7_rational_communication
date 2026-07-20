//! # Error Types
//!
//! Unified error enumeration for the entire application.
//! Uses `thiserror` to derive `std::error::Error` with minimal boilerplate.
//!
//! ## Design
//! We define a single `AppError` enum that wraps all fallible operations:
//! - NFC / serial-port errors from the PN532
//! - Database errors from SQLite
//! - Display / rendering errors
//! - Configuration parsing errors
//! - I/O errors
//!
//! This lets callers use `Result<T>` (aliased as `AppResult<T>`) everywhere
//! without dealing with five different error types.

use thiserror::Error;

/// Alias for `Result<T, AppError>` — the app-wide return type for fallible operations.
pub type AppResult<T> = Result<T, AppError>;

/// All errors that can occur in the NearField system.
#[derive(Error, Debug)]
pub enum AppError {
    // ── NFC / PN532 ──────────────────────────────────────────────────────

    /// The serial port could not be opened or is not available.
    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),

    /// Communication with the PN532 failed (timeout, bad checksum, NAK, etc.).
    #[error("PN532 communication error: {0}")]
    Pn532Protocol(String),

    /// No NFC tag found in the RF field after a poll cycle.
    #[error("No tag detected")]
    NoTag,

    /// A tag was detected but its type or UID is not supported.
    #[error("Unsupported tag: {0}")]
    UnsupportedTag(String),

    /// The NFC antenna returned an unexpected firmware / status byte.
    #[error("PN532 unexpected response: {0}")]
    Pn532Unexpected(String),

    // ── Database ─────────────────────────────────────────────────────────

    /// SQLite operation failed (connection, query, migration, etc.).
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    // ── Display / Render ─────────────────────────────────────────────────

    /// The display device could not be initialised or written to.
    #[error("Display error: {0}")]
    Display(String),

    // ── Configuration ────────────────────────────────────────────────────

    /// Config file is missing, malformed, or contains invalid values.
    #[error("Configuration error: {0}")]
    Config(String),

    // ── General I/O ──────────────────────────────────────────────────────

    /// General filesystem / I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    // ── Catch-all ────────────────────────────────────────────────────────

    /// Any other unforseen error.
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
