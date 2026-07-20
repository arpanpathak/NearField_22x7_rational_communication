use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Serial port error: {0}")]
    Serial(#[from] serialport::Error),

    #[error("PN532 communication error: {0}")]
    Pn532Protocol(String),

    #[error("No tag detected")]
    NoTag,

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("Display error: {0}")]
    Display(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

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
