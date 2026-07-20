use crate::error::AppResult;
use crate::display::DisplayBackend;

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
