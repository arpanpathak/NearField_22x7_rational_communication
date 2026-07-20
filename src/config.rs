//! # Configuration
//!
//! Loads settings from a TOML config file (`nearfield.toml`) or environment
//! variables.  The config struct holds:
//!
//! - **Serial port** path (e.g. `/dev/ttyAMA0` on Pi Zero 2WH UART)
//! - **Database** path (where the SQLite file lives)
//! - **Poll interval** (how often to check for new tags, in milliseconds)
//! - **Display** type and optional device parameters
//!
//! ## Lookup order
//! 1. `NEARFIELD_CONFIG` env var pointing to a config file path
//! 2. `./nearfield.toml` in the working directory
//! 3. `/etc/nearfield/nearfield.toml`
//! 4. Hard-coded defaults (see [`Config::default`])
//!
//! ## Environment variable overrides
//! Every field can be overridden at runtime:
//! - `NEARFIELD_SERIAL_PORT`
//! - `NEARFIELD_DB_PATH`
//! - `NEARFIELD_POLL_MS`
//! - `NEARFIELD_DISPLAY_TYPE`

use std::path::PathBuf;

use serde::Deserialize;

use crate::error::{AppError, AppResult};

/// Top-level application configuration.
///
/// Deserialised from TOML; all fields have sensible defaults.
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Serial port path for PN532 (e.g. `/dev/ttyAMA0`, `/dev/serial0`).
    pub serial_port: String,

    /// Baud rate for the UART connection (PN532 HSU default is 115200).
    pub serial_baud: u32,

    /// Path to the SQLite database file.
    pub db_path: PathBuf,

    /// How often (in milliseconds) to poll for a new NFC tag.
    pub poll_interval_ms: u64,

    /// Display backend to use: `"none"`, `"stdout"` (debug), or `"eink"`.
    pub display_type: String,

    /// If set, the path to a FIFO / pipe for the display renderer.
    pub display_pipe: Option<PathBuf>,

    /// Maximum number of log entries to keep in DB (0 = unlimited).
    pub max_log_entries: u64,

    /// Verbosity level (0 = default, 1 = debug, 2 = trace).
    pub verbosity: u8,
}

impl Default for Config {
    /// Sensible defaults for a Raspberry Pi Zero 2WH + PN532 over UART.
    fn default() -> Self {
        Self {
            serial_port: "/dev/ttyAMA0".into(),
            serial_baud: 115_200,
            db_path: PathBuf::from("./nearfield.db"),
            poll_interval_ms: 500,   // Poll twice a second — low-power friendly
            display_type: "stdout".into(), // Default to stdout for debugging
            display_pipe: None,
            max_log_entries: 10_000,
            verbosity: 0,
        }
    }
}

impl Config {
    /// Load configuration by searching known paths and applying env overrides.
    ///
    /// Returns the merged [`Config`] or an error if a specified file exists
    /// but cannot be parsed.
    pub fn load() -> AppResult<Self> {
        // 1. Determine config file path from env or default locations.
        let config_path = std::env::var("NEARFIELD_CONFIG")
            .ok()
            .map(PathBuf::from)
            .or_else(|| {
                // Try local first, then /etc
                let local = PathBuf::from("./nearfield.toml");
                if local.exists() {
                    return Some(local);
                }
                let etc = PathBuf::from("/etc/nearfield/nearfield.toml");
                if etc.exists() {
                    return Some(etc);
                }
                None
            });

        // 2. Start with defaults.
        let mut config: Config = if let Some(path) = config_path {
            let contents = std::fs::read_to_string(&path)
                .map_err(|e| AppError::Config(format!("Cannot read {path:?}: {e}")))?;
            toml::from_str(&contents)
                .map_err(|e| AppError::Config(format!("Invalid TOML in {path:?}: {e}")))?
        } else {
            Config::default()
        };

        // 3. Apply environment overrides.
        if let Ok(port) = std::env::var("NEARFIELD_SERIAL_PORT") {
            config.serial_port = port;
        }
        if let Ok(baud) = std::env::var("NEARFIELD_SERIAL_BAUD") {
            config.serial_baud = baud.parse::<u32>()
                .map_err(|e| AppError::Config(format!("Invalid NEARFIELD_SERIAL_BAUD: {e}")))?;
        }
        if let Ok(db) = std::env::var("NEARFIELD_DB_PATH") {
            config.db_path = PathBuf::from(db);
        }
        if let Ok(ms) = std::env::var("NEARFIELD_POLL_MS") {
            config.poll_interval_ms = ms.parse::<u64>()
                .map_err(|e| AppError::Config(format!("Invalid NEARFIELD_POLL_MS: {e}")))?;
        }
        if let Ok(dt) = std::env::var("NEARFIELD_DISPLAY_TYPE") {
            config.display_type = dt;
        }

        Ok(config)
    }
}
