//! # NearField_22x7_rational_communication
//!
//! A low-power NFC tag reader for Raspberry Pi Zero 2WH.
//!
//! ## Pipeline
//!
//! ```text
//! ┌─────────┐   ┌──────────┐   ┌──────────┐   ┌───────────┐
//! │  PN532  │ → │  Poll()  │ → │  SQLite  │ → │  Display  │
//! │ (UART)  │   │ TagInfo  │   │  Logger  │   │ 22×7 ASC  │
//! └─────────┘   └──────────┘   └──────────┘   └───────────┘
//! ```
//!
//! ## Run
//!
//! ```bash
//! RUST_LOG=info cargo run --release
//! ```
//!
//! ## Environment Variables
//!
//! | Variable              | Default           | Description             |
//! |-----------------------|-------------------|-------------------------|
//! | `NEARFIELD_SERIAL_PORT` | `/dev/ttyAMA0`  | PN532 serial port path  |
//! | `NEARFIELD_DB_PATH`     | `./nearfield.db` | SQLite database path    |
//! | `NEARFIELD_POLL_MS`     | `500`            | Poll interval (ms)      |
//! | `NEARFIELD_DISPLAY_TYPE`| `stdout`         | Display backend         |
//! | `RUST_LOG`              | —                | Log level               |

mod config;
mod display;
mod error;
mod nfc;
mod storage;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use crate::config::Config;
use crate::display::{AsciiRenderer, DisplayBackend, backend_from_config};
use crate::error::{AppError, AppResult};
use crate::nfc::Pn532;
use crate::storage::Database;

fn main() -> AppResult<()> {
    // ── Initialise logging ──────────────────────────────────────────────
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_secs()
        .init();

    log::info!(
        "⟐ NearField_22x7_rational_communication v{}",
        env!("CARGO_PKG_VERSION")
    );

    // ── Load configuration ─────────────────────────────────────────────
    let config = Config::load()?;
    log::info!("Config: serial={port}, db={db}, poll={poll}ms, display={disp}",
        port = config.serial_port,
        db = config.db_path.display(),
        poll = config.poll_interval_ms,
        disp = config.display_type,
    );

    // ── Initialise display ─────────────────────────────────────────────
    let mut display: Box<dyn DisplayBackend> =
        backend_from_config(&config.display_type, config.display_pipe.clone())?;
    display.init()?;

    // ── Initialise database ─────────────────────────────────────────────
    let db = Database::open(&config.db_path)?;
    if config.max_log_entries > 0 {
        db.trim_to(config.max_log_entries)?;
    }

    // ── Initialise PN532 ───────────────────────────────────────────────
    let mut pn532 = Pn532::open(&config.serial_port, config.serial_baud)?;

    // Verify the chip is alive.
    match pn532.get_firmware_version() {
        Ok(ver) => log::info!("Firmware: {ver}"),
        Err(e) => {
            log::error!("PN532 not responding: {e}");
            log::error!(
                "Check wiring: PN532 TX→Pi RX (GP15), PN532 RX→Pi TX (GP14), \
                 VCC→3.3V, GND→GND"
            );
            log::error!("Also ensure UART is enabled: sudo raspi-config → Interface → Serial Port");
            return Err(e);
        }
    }

    // Configure SAM (Security Access Module) for tag reading.
    pn532.sam_config()?;

    // ── Set up signal handler for graceful shutdown ────────────────────
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        log::info!("Received Ctrl-C, shutting down...");
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| AppError::Other(format!("Cannot set Ctrl-C handler: {e}")))?;

    // ── ASCII renderer ─────────────────────────────────────────────────
    let renderer = AsciiRenderer;
    let mut tap_count: u64 = 0;

    // Show idle screen on startup.
    let idle_frame = renderer.render_idle();
    display.display_frame(&idle_frame)?;

    // ── Main Polling Loop ───────────────────────────────────────────────
    log::info!("Entering main loop (poll every {} ms)", config.poll_interval_ms);

    while running.load(Ordering::SeqCst) {
        match pn532.poll() {
            Ok(tag) => {
                tap_count += 1;
                log::info!("{tag}");

                // ── Log to database ─────────────────────────────────────
                match db.insert_tag(&tag) {
                    Ok(id) => log::debug!("Inserted row id={id}"),
                    Err(e) => log::error!("DB insert failed: {e}"),
                }

                // ── Render to display ───────────────────────────────────
                let frame = renderer.render_tag(&tag, tap_count);
                if let Err(e) = display.display_frame(&frame) {
                    log::error!("Display error: {e}");
                }

                // ── Brief hold to show the tag info ─────────────────────
                // Keep the tag info on screen for a moment before resuming
                // polling, so the user can read it.
                std::thread::sleep(Duration::from_secs(2));

                // Show idle again after the hold.
                let idle = renderer.render_idle();
                display.display_frame(&idle)?;
            }
            Err(AppError::NoTag) => {
                // No tag in field — that's normal, just keep polling.
                std::thread::sleep(Duration::from_millis(config.poll_interval_ms));
            }
            Err(e) => {
                log::error!("Poll error: {e}");
                // If we lose serial communication, wait a bit and retry.
                std::thread::sleep(Duration::from_secs(2));

                // Try to re-initialise the PN532.
                log::info!("Attempting PN532 re-initialisation...");
                match Pn532::open(&config.serial_port, config.serial_baud) {
                    Ok(new_pn532) => {
                        // Reconfigure SAM on the new connection.
                        pn532 = new_pn532;
                        if let Err(sam_err) = pn532.sam_config() {
                            log::error!("Re-init SAM failed: {sam_err}");
                        } else {
                            log::info!("PN532 re-initialised successfully");
                        }
                    }
                    Err(reopen_err) => {
                        log::error!("Re-open failed: {reopen_err}");
                    }
                }
            }
        }
    }

    // ── Shutdown ────────────────────────────────────────────────────────
    log::info!("Shutting down. Total tags logged: {tap_count}");
    display.clear()?;

    // Print a summary.
    match db.total_count() {
        Ok(count) => log::info!("Database contains {count} total entries"),
        Err(e) => log::error!("Failed to get DB count: {e}"),
    }

    Ok(())
}
