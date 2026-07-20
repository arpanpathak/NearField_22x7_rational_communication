use std::io::Write;
use crate::error::AppResult;
use crate::display::DisplayBackend;

pub struct StdoutBackend {
    pub use_ansi_clear: bool,
}

impl StdoutBackend {
    pub fn new() -> Self {
        Self { use_ansi_clear: true }
    }
}

impl DisplayBackend for StdoutBackend {
    fn init(&mut self) -> AppResult<()> {
        log::info!("Display backend: stdout");
        Ok(())
    }

    fn display_frame(&mut self, frame: &[String]) -> AppResult<()> {
        if self.use_ansi_clear {
            print!("\x1B[2J\x1B[H");
        }
        for line in frame {
            println!("{}", line);
        }
        std::io::stdout().flush().ok();
        Ok(())
    }

    fn clear(&mut self) -> AppResult<()> {
        if self.use_ansi_clear {
            print!("\x1B[2J\x1B[H");
            std::io::stdout().flush().ok();
        }
        Ok(())
    }
}
