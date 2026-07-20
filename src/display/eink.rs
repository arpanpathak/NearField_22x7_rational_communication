/// E-ink display backend (stub).
///
/// Placeholder for Waveshare SPI e-ink display support. The actual
/// SPI driver requires `embedded-hal`, `linux-embedded-hal`, and
/// `epd-waveshare` crates. This stub logs calls and returns `Ok`.
///
/// # To implement
/// 1. Add `embedded-hal`, `linux-embedded-hal`, `epd-waveshare` to `Cargo.toml`
/// 2. Initialise SPI via `/dev/spidev0.0`
/// 3. Render the ASCII frame to a bitmap and call the EPD driver

use crate::error::AppResult;
use crate::display::DisplayBackend;

/// E-ink display backend (stub implementation).
pub struct EinkBackend;

impl EinkBackend {
    pub fn new() -> Self {
        Self
    }
}

impl DisplayBackend for EinkBackend {
    fn init(&mut self) -> AppResult<()> {
        log::info!("Display backend: eink (stub)");
        log::warn!("EinkBackend is a stub. Implement SPI HAL for your display.");
        Ok(())
    }

    fn display_frame(&mut self, _frame: &[String]) -> AppResult<()> {
        log::debug!("EinkBackend.display_frame() - no-op stub");
        Ok(())
    }

    fn clear(&mut self) -> AppResult<()> {
        log::debug!("EinkBackend.clear() - no-op stub");
        Ok(())
    }
}
