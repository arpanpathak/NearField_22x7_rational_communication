/// Terminal display backend.
///
/// Prints each 22x7 ASCII frame to stdout, with optional ANSI clear
/// sequences to create a refresh effect. Useful for debugging on a
/// desktop machine or over SSH without physical display hardware.

use std::io::Write;
use crate::error::AppResult;
use crate::display::DisplayBackend;

/// ANSI escape sequence: clear entire screen.
const ANSI_CLEAR: &str = "\x1B[2J\x1B[H";

/// Display backend that prints frames to stdout.
pub struct StdoutBackend {
    /// Whether to clear the terminal before each frame.
    pub use_ansi_clear: bool,
}

impl StdoutBackend {
    /// Create a new stdout backend with ANSI clearing enabled.
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
            print!("{}", ANSI_CLEAR);
        }
        for line in frame {
            println!("{}", line);
        }
        std::io::stdout().flush().ok();
        Ok(())
    }

    fn clear(&mut self) -> AppResult<()> {
        if self.use_ansi_clear {
            print!("{}", ANSI_CLEAR);
            std::io::stdout().flush().ok();
        }
        Ok(())
    }
}
