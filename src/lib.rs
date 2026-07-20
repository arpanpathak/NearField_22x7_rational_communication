/// NearField_22x7_rational_communication library crate.
///
/// This is the main library entry point. It re-exports all public modules
/// and provides the top-level [`run`] function that initialises hardware,
/// enters the polling loop, and handles shutdown.

pub mod config;
pub mod display;
pub mod error;
pub mod nfc;
pub mod pipeline;
pub mod storage;

pub use error::AppResult;

use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::config::Config;
use crate::display::{backend_from_config, AsciiRenderer, DisplayBackend};
use crate::error::AppError;
use crate::nfc::Pn532;
use crate::pipeline::shutdown::setup_signal_handler;
use crate::storage::Database;

/// How long to display a detected tag before returning to idle.
const TAG_DISPLAY_SECONDS: u64 = 2;

/// How long to wait after an error before attempting reconnection.
const ERROR_BACKOFF_SECONDS: u64 = 2;

/// Initialise all subsystems and enter the main polling loop.
///
/// # Pipeline
/// 1. Load config from file/env
/// 2. Initialise display backend
/// 3. Open SQLite database (auto-migrate)
/// 4. Open PN532 serial port, verify firmware, configure SAM
/// 5. Install Ctrl-C handler
/// 6. Loop: poll for tags -> log to DB -> render to display
/// 7. On shutdown: clear display, print summary
pub fn run() -> AppResult<()> {
    let config = Config::load()?;

    let mut display: Box<dyn DisplayBackend> =
        backend_from_config(&config.display_type, config.display_pipe.clone())?;
    display.init()?;

    let db = Database::open(&config.db_path)?;
    if config.max_log_entries > 0 {
        db.trim_to(config.max_log_entries)?;
    }

    let mut pn532 = Pn532::open(&config.serial_port, config.serial_baud)?;

    match pn532.get_firmware_version() {
        Ok(ver) => log::info!("Firmware: {ver}"),
        Err(e) => {
            log::error!("PN532 not responding: {e}");
            log::error!("Check wiring: PN532 TX->Pi RX (GP15), PN532 RX->Pi TX (GP14), VCC->3.3V, GND->GND");
            log::error!("Enable UART: sudo raspi-config -> Interface -> Serial Port");
            return Err(e);
        }
    }

    pn532.sam_config()?;

    let running = setup_signal_handler()?;
    let renderer = AsciiRenderer;
    let mut tap_count: u64 = 0;

    display.display_frame(&renderer.render_idle())?;
    log::info!("Entering main loop (poll every {} ms)", config.poll_interval_ms);

    while running.load(Ordering::SeqCst) {
        match pn532.poll() {
            Ok(tag) => {
                tap_count += 1;
                log::info!("{tag}");

                if let Err(e) = db.insert_tag(&tag) {
                    log::error!("DB insert failed: {e}");
                }

                let frame = renderer.render_tag(&tag, tap_count);
                if let Err(e) = display.display_frame(&frame) {
                    log::error!("Display error: {e}");
                }

                std::thread::sleep(Duration::from_secs(TAG_DISPLAY_SECONDS));

                if let Err(e) = display.display_frame(&renderer.render_idle()) {
                    log::error!("Display error: {e}");
                }
            }
            Err(AppError::NoTag) => {
                std::thread::sleep(Duration::from_millis(config.poll_interval_ms));
            }
            Err(e) => {
                log::error!("Poll error: {e}");
                std::thread::sleep(Duration::from_secs(ERROR_BACKOFF_SECONDS));
                pn532 = try_reconnect(&config);
            }
        }
    }

    log::info!("Shutting down. Total tags logged: {tap_count}");
    display.clear()?;
    if let Ok(count) = db.total_count() {
        log::info!("Database contains {count} total entries");
    }

    Ok(())
}

/// Attempt to re-open the serial port and re-configure the PN532.
fn try_reconnect(config: &Config) -> Pn532 {
    log::info!("Attempting PN532 re-initialisation...");
    match Pn532::open(&config.serial_port, config.serial_baud) {
        Ok(mut pn532) => {
            if let Err(e) = pn532.sam_config() {
                log::error!("Re-init SAM failed: {e}");
            } else {
                log::info!("PN532 re-initialised successfully");
            }
            pn532
        }
        Err(e) => {
            log::error!("Re-open failed: {e}");
            panic!("Cannot recover serial connection: {e}");
        }
    }
}
