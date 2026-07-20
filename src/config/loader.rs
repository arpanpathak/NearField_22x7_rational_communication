/// Configuration file loading and environment variable overrides.
///
/// This module separates the *how* of config loading from the *what*
/// (the [`Config`] struct). It searches standard paths for a TOML file,
/// then applies environment variable overrides on top.

use std::path::PathBuf;
use crate::config::Config;
use crate::error::{AppError, AppResult};

/// Environment variable that overrides the config file path.
const ENV_CONFIG_PATH: &str = "NEARFIELD_CONFIG";

/// Per-field environment variable names for runtime overrides.
const ENV_SERIAL_PORT: &str = "NEARFIELD_SERIAL_PORT";
const ENV_SERIAL_BAUD: &str = "NEARFIELD_SERIAL_BAUD";
const ENV_DB_PATH: &str = "NEARFIELD_DB_PATH";
const ENV_POLL_MS: &str = "NEARFIELD_POLL_MS";
const ENV_DISPLAY_TYPE: &str = "NEARFIELD_DISPLAY_TYPE";

/// Local directory config file path.
const CONFIG_FILE_LOCAL: &str = "./nearfield.toml";
/// System-wide config file path.
const CONFIG_FILE_ETC: &str = "/etc/nearfield/nearfield.toml";

/// Load configuration from file (if found) and apply env overrides.
pub fn load() -> AppResult<Config> {
    let config_path = find_config_file();
    let mut config: Config = if let Some(path) = config_path {
        let contents = std::fs::read_to_string(&path)
            .map_err(|e| AppError::Config(format!("Cannot read {:?}: {}", path, e)))?;
        toml::from_str(&contents)
            .map_err(|e| AppError::Config(format!("Invalid TOML in {:?}: {}", path, e)))?
    } else {
        Config::default()
    };
    apply_env_overrides(&mut config)?;
    Ok(config)
}

/// Search for a config file in priority order.
fn find_config_file() -> Option<PathBuf> {
    if let Ok(path) = std::env::var(ENV_CONFIG_PATH) {
        return Some(PathBuf::from(path));
    }
    let local = PathBuf::from(CONFIG_FILE_LOCAL);
    if local.exists() {
        return Some(local);
    }
    let etc = PathBuf::from(CONFIG_FILE_ETC);
    if etc.exists() {
        return Some(etc);
    }
    None
}

/// Override config fields from environment variables.
///
/// Uses a macro to reduce repetition for simple string fields.
fn apply_env_overrides(config: &mut Config) -> AppResult<()> {
    /// Apply a string-type env override if the variable is set.
    macro_rules! env_str {
        ($var:expr, $field:expr) => {
            if let Ok(val) = std::env::var($var) {
                $field = val.into();
            }
        };
    }

    env_str!(ENV_SERIAL_PORT, config.serial_port);
    env_str!(ENV_DB_PATH, config.db_path);
    env_str!(ENV_DISPLAY_TYPE, config.display_type);

    if let Ok(baud) = std::env::var(ENV_SERIAL_BAUD) {
        config.serial_baud = baud
            .parse::<u32>()
            .map_err(|e| AppError::Config(format!("Invalid {}: {}", ENV_SERIAL_BAUD, e)))?;
    }

    if let Ok(ms) = std::env::var(ENV_POLL_MS) {
        config.poll_interval_ms = ms
            .parse::<u64>()
            .map_err(|e| AppError::Config(format!("Invalid {}: {}", ENV_POLL_MS, e)))?;
    }

    Ok(())
}
