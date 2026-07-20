/// Configuration management for the NearField NFC reader.
///
/// The [`Config`] struct holds all user-settable parameters and is loaded
/// from a combination of a TOML config file and environment variable overrides.
///
/// # Lookup order
/// 1. `NEARFIELD_CONFIG` env var pointing to a config file path
/// 2. `./nearfield.toml` in the working directory
/// 3. `/etc/nearfield/nearfield.toml`
/// 4. Hard-coded defaults (see [`Config::default`])
///
/// Every field can be overridden at runtime via environment variables.
/// See [`loader`] for details.

pub mod loader;

use std::path::PathBuf;
use serde::Deserialize;

/// All user-configurable parameters for the NearField system.
///
/// Deserialised from TOML via serde. All fields have sensible defaults
/// so the application runs out of the box without a config file.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Serial port device path for the PN532 (e.g. `/dev/ttyAMA0`).
    pub serial_port: String,

    /// UART baud rate for PN532 communication.
    pub serial_baud: u32,

    /// File path for the SQLite database.
    pub db_path: PathBuf,

    /// Milliseconds between NFC poll cycles. Lower = more responsive,
    /// higher = lower power consumption.
    pub poll_interval_ms: u64,

    /// Display backend to use. One of: `"stdout"`, `"eink"`, `"pipe"`, `"none"`.
    pub display_type: String,

    /// Path to a named pipe (FIFO) for the pipe display backend.
    pub display_pipe: Option<PathBuf>,

    /// Maximum log entries before oldest records are trimmed.
    /// Set to `0` to disable trimming.
    pub max_log_entries: u64,
}

impl Default for Config {
    /// Sensible defaults targeting a Raspberry Pi Zero 2WH with PN532 over UART.
    fn default() -> Self {
        Self {
            serial_port: "/dev/ttyAMA0".into(),
            serial_baud: 115_200,
            db_path: PathBuf::from("./nearfield.db"),
            poll_interval_ms: 500,
            display_type: "stdout".into(),
            display_pipe: None,
            max_log_entries: 10_000,
        }
    }
}

impl Config {
    /// Load configuration by searching default paths and applying env overrides.
    ///
    /// This delegates to [`loader::load`] for the actual logic, keeping
    /// this module focused on the data model.
    pub fn load() -> crate::error::AppResult<Self> {
        loader::load()
    }
}
