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
            log::error!(
                "Check wiring: PN532 TX->Pi RX (GP15), PN532 RX->Pi TX (GP14), VCC->3.3V, GND->GND"
            );
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

                std::thread::sleep(Duration::from_secs(2));

                if let Err(e) = display.display_frame(&renderer.render_idle()) {
                    log::error!("Display error: {e}");
                }
            }
            Err(AppError::NoTag) => {
                std::thread::sleep(Duration::from_millis(config.poll_interval_ms));
            }
            Err(e) => {
                log::error!("Poll error: {e}");
                std::thread::sleep(Duration::from_secs(2));
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
            unreachable!()
        }
    }
}
