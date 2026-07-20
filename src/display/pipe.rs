/// Named-pipe display backend.
///
/// Writes each 22x7 frame as a JSON object to a UNIX named pipe (FIFO).
/// External processes (e.g. a Python e-ink script) can read from the pipe
/// and render the frame on the actual display hardware.
///
/// # JSON format
/// ```json
/// {"type":"frame","width":22,"height":7,"lines":["+--+", ...]}
/// {"type":"clear"}
/// ```

use std::path::PathBuf;
use std::fs::OpenOptions;
use std::io::Write;
use crate::error::{AppError, AppResult};
use crate::display::DisplayBackend;

/// Named-pipe display backend.
pub struct PipeBackend {
    pipe_path: PathBuf,
}

impl PipeBackend {
    /// Create a new pipe backend.
    ///
    /// The FIFO will be created if it does not exist.
    pub fn new(pipe_path: PathBuf) -> Self {
        Self { pipe_path }
    }
}

impl DisplayBackend for PipeBackend {
    fn init(&mut self) -> AppResult<()> {
        if !self.pipe_path.exists() {
            let status = std::process::Command::new("mkfifo")
                .arg(self.pipe_path.as_os_str())
                .status()
                .map_err(|e| AppError::Display(format!("mkfifo failed: {}", e)))?;
            if !status.success() {
                return Err(AppError::Display("mkfifo returned non-zero exit status".into()));
            }
            log::info!("Created FIFO at {}", self.pipe_path.display());
        }
        log::info!("Display backend: pipe ({})", self.pipe_path.display());
        Ok(())
    }

    fn display_frame(&mut self, frame: &[String]) -> AppResult<()> {
        let payload = serde_json::json!({
            "type": "frame",
            "width": 22,
            "height": 7,
            "lines": frame,
        });
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.pipe_path)
            .map_err(|e| AppError::Display(format!("Cannot open pipe: {}", e)))?;
        writeln!(file, "{}", payload.to_string())
            .map_err(|e| AppError::Display(format!("Pipe write error: {}", e)))?;
        file.flush().ok();
        Ok(())
    }

    fn clear(&mut self) -> AppResult<()> {
        let payload = serde_json::json!({ "type": "clear" });
        if let Ok(mut file) = OpenOptions::new().write(true).open(&self.pipe_path) {
            writeln!(file, "{}", payload.to_string()).ok();
        }
        Ok(())
    }
}
