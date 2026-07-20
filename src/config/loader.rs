use std::path::PathBuf;
use crate::config::Config;
use crate::error::{AppError, AppResult};

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

fn find_config_file() -> Option<PathBuf> {
    if let Ok(path) = std::env::var("NEARFIELD_CONFIG") {
        return Some(PathBuf::from(path));
    }
    let local = PathBuf::from("./nearfield.toml");
    if local.exists() {
        return Some(local);
    }
    let etc = PathBuf::from("/etc/nearfield/nearfield.toml");
    if etc.exists() {
        return Some(etc);
    }
    None
}

fn apply_env_overrides(config: &mut Config) -> AppResult<()> {
    macro_rules! env_override {
        ($var:literal, $field:expr) => {
            if let Ok(val) = std::env::var($var) {
                $field = val.into();
            }
        };
    }

    env_override!("NEARFIELD_SERIAL_PORT", config.serial_port);

    if let Ok(baud) = std::env::var("NEARFIELD_SERIAL_BAUD") {
        config.serial_baud = baud.parse::<u32>()
            .map_err(|e| AppError::Config(format!("Invalid NEARFIELD_SERIAL_BAUD: {}", e)))?;
    }

    env_override!("NEARFIELD_DB_PATH", config.db_path);

    if let Ok(ms) = std::env::var("NEARFIELD_POLL_MS") {
        config.poll_interval_ms = ms.parse::<u64>()
            .map_err(|e| AppError::Config(format!("Invalid NEARFIELD_POLL_MS: {}", e)))?;
    }

    env_override!("NEARFIELD_DISPLAY_TYPE", config.display_type);

    Ok(())
}
