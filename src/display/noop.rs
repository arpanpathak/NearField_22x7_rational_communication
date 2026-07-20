/// No-op display backend.
///
/// Accepts all calls silently. Use `display_type = "none"` to disable
/// display output entirely (e.g. for headless or batch operation).

use crate::error::AppResult;
use crate::display::DisplayBackend;

/// A display backend that does nothing.
pub struct NoopBackend;

impl DisplayBackend for NoopBackend {
    fn init(&mut self) -> AppResult<()> {
        Ok(())
    }
    fn display_frame(&mut self, _: &[String]) -> AppResult<()> {
        Ok(())
    }
    fn clear(&mut self) -> AppResult<()> {
        Ok(())
    }
}
