use crate::error::AppResult;
use crate::display::DisplayBackend;

pub struct NoopBackend;

impl DisplayBackend for NoopBackend {
    fn init(&mut self) -> AppResult<()> { Ok(()) }
    fn display_frame(&mut self, _: &[String]) -> AppResult<()> { Ok(()) }
    fn clear(&mut self) -> AppResult<()> { Ok(()) }
}
