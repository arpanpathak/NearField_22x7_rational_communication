pub mod loader;

use std::path::PathBuf;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct Config {
    /// Serial port path (e.g. /dev/ttyAMA0)
    pub serial_port: String,
    /// UART baud rate
    pub serial_baud: u32,
    /// SQLite database file path
    pub db_path: PathBuf,
    /// Poll interval in milliseconds
    pub poll_interval_ms: u64,
    /// Display backend type: stdout, eink, pipe, none
    pub display_type: String,
    /// Named pipe path for pipe backend
    pub display_pipe: Option<PathBuf>,
    /// Max log entries before trimming (0 = unlimited)
    pub max_log_entries: u64,
}

impl Default for Config {
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
    pub fn load() -> crate::error::AppResult<Self> {
        loader::load()
    }
}
