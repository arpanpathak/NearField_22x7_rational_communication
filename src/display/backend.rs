//! # Display Backends
//!
//! Defines the [`DisplayBackend`] trait and concrete implementations.
//!
//! ## Available Backends
//!
//! | Backend         | Config Value | Description                        |
//! |-----------------|--------------|------------------------------------|
//! | `StdoutBackend` | `"stdout"`   | Print to stdout (debugging)        |
//! | `EinkBackend`   | `"eink"`     | Waveshare e-ink display via SPI    |
//! | `PipeBackend`   | `"pipe"`     | Write to a FIFO / named pipe       |
//!
//! ## Adding a New Backend
//!
//! 1. Implement the [`DisplayBackend`] trait.
//! 2. Add a variant to the match in [`backend_from_config`].

use std::path::PathBuf;
use std::fs::{File, OpenOptions};
use std::io::Write;

use crate::error::{AppError, AppResult};

/// Trait for rendering a frame (22×7 ASCII) to a physical or virtual display.
///
/// The `frame` argument is always exactly 7 lines, each 22 characters wide.
pub trait DisplayBackend {
    /// Initialise the display (open device, configure, etc.).
    fn init(&mut self) -> AppResult<()>;

    /// Write a complete 22×7 frame to the display.
    fn display_frame(&mut self, frame: &[String]) -> AppResult<()>;

    /// Clear the display.
    fn clear(&mut self) -> AppResult<()>;

    /// Human-readable name for logging.
    fn name(&self) -> &str;
}

// ── StdoutBackend ───────────────────────────────────────────────────────

/// Prints each frame to stdout, optionally with ANSI clear.
///
/// Useful for debugging on a desktop machine without physical hardware.
pub struct StdoutBackend {
    /// If true, clears the terminal before each frame.
    pub use_ansi_clear: bool,
}

impl StdoutBackend {
    pub fn new() -> Self {
        Self {
            use_ansi_clear: true,
        }
    }
}

impl DisplayBackend for StdoutBackend {
    fn init(&mut self) -> AppResult<()> {
        log::info!("Display backend: stdout");
        Ok(())
    }

    fn display_frame(&mut self, frame: &[String]) -> AppResult<()> {
        if self.use_ansi_clear {
            // ANSI escape: clear screen, move cursor home.
            print!("\x1B[2J\x1B[H");
        }
        for line in frame {
            println!("{line}");
        }
        // Flush stdout so the output is immediate (important over SSH).
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

    fn name(&self) -> &str {
        "stdout"
    }
}

// ── PipeBackend ─────────────────────────────────────────────────────────

/// Writes frames to a named pipe (FIFO) as newline-delimited JSON.
///
/// External processes (e.g. a Python e-ink script) can read from the pipe
/// and render the frame on the actual display hardware.
pub struct PipeBackend {
    pipe_path: PathBuf,
    file: Option<File>,
}

impl PipeBackend {
    /// Create a new pipe backend.
    ///
    /// If the pipe doesn't exist, it will be created with `mkfifo`.
    pub fn new(pipe_path: PathBuf) -> Self {
        Self {
            pipe_path,
            file: None,
        }
    }
}

impl DisplayBackend for PipeBackend {
    fn init(&mut self) -> AppResult<()> {
        // Create the FIFO if it doesn't exist.
        if !self.pipe_path.exists() {
            // mkfifo on Unix
            let path_str = self.pipe_path.to_string_lossy();
            let status = std::process::Command::new("mkfifo")
                .arg(&path_str as &str)
                .status()
                .map_err(|e| AppError::Display(format!("mkfifo failed: {e}")))?;

            if !status.success() {
                return Err(AppError::Display(
                    "mkfifo returned non-zero exit status".into(),
                ));
            }
            log::info!("Created FIFO at {}", self.pipe_path.display());
        }

        log::info!("Display backend: pipe ({})", self.pipe_path.display());
        Ok(())
    }

    fn display_frame(&mut self, frame: &[String]) -> AppResult<()> {
        let frame_json = serde_json::json!({
            "type": "frame",
            "width": 22,
            "height": 7,
            "lines": frame,
        });

        // Open the pipe (blocks until a reader connects).
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.pipe_path)
            .map_err(|e| AppError::Display(format!("Cannot open pipe: {e}")))?;

        writeln!(file, "{}", frame_json.to_string())
            .map_err(|e| AppError::Display(format!("Pipe write error: {e}")))?;

        file.flush().ok();
        Ok(())
    }

    fn clear(&mut self) -> AppResult<()> {
        let blank = serde_json::json!({
            "type": "clear",
        });
        let mut file = OpenOptions::new()
            .write(true)
            .open(&self.pipe_path)
            .map_err(|e| AppError::Display(format!("Cannot open pipe: {e}")))?;
        writeln!(file, "{}", blank.to_string()).ok();
        Ok(())
    }

    fn name(&self) -> &str {
        "pipe"
    }
}

// ── EinkBackend (stub) ──────────────────────────────────────────────────

/// E-ink display backend (placeholder / stub).
///
/// Actual SPI drivers for Waveshare displays require `embedded-hal` and
/// `linux-embedded-hal` crates. This implementation provides the trait
/// interface; the SPI HAL layer is left as an exercise for the user's
/// specific display model.
///
/// ## To implement:
/// 1. Add `embedded-hal`, `linux-embedded-hal`, and `epd-waveshare` to
///    `Cargo.toml`.
/// 2. Initialise SPI via `/dev/spidev0.0`.
/// 3. Call the EPD driver's `display_frame()` with a bitmap rendered from
///    the ASCII grid.
pub struct EinkBackend;

impl EinkBackend {
    pub fn new() -> Self {
        Self
    }
}

impl DisplayBackend for EinkBackend {
    fn init(&mut self) -> AppResult<()> {
        log::info!("Display backend: eink (stub — not yet wired to SPI)");
        log::warn!("EinkBackend is a stub. Implement SPI HAL for your display.");
        Ok(())
    }

    fn display_frame(&mut self, _frame: &[String]) -> AppResult<()> {
        // TODO: render ASCII frame to e-ink bitmap and send via SPI.
        log::debug!("EinkBackend.display_frame() called — no-op stub");
        Ok(())
    }

    fn clear(&mut self) -> AppResult<()> {
        log::debug!("EinkBackend.clear() called — no-op stub");
        Ok(())
    }

    fn name(&self) -> &str {
        "eink"
    }
}

// ── Factory ─────────────────────────────────────────────────────────────

/// Construct a display backend from a configuration string.
///
/// # Errors
/// Returns `AppError::Config` if the backend type is unknown.
pub fn backend_from_config(display_type: &str, pipe_path: Option<PathBuf>) -> AppResult<Box<dyn DisplayBackend>> {
    match display_type {
        "stdout" => Ok(Box::new(StdoutBackend::new())),
        "eink" => Ok(Box::new(EinkBackend::new())),
        "pipe" => {
            let path = pipe_path
                .ok_or_else(|| AppError::Config("pipe backend requires display_pipe path".into()))?;
            Ok(Box::new(PipeBackend::new(path)))
        }
        "none" => {
            log::info!("Display disabled (type=none)");
            // A no-op backend that does nothing.
            struct NoopBackend;
            impl DisplayBackend for NoopBackend {
                fn init(&mut self) -> AppResult<()> { Ok(()) }
                fn display_frame(&mut self, _: &[String]) -> AppResult<()> { Ok(()) }
                fn clear(&mut self) -> AppResult<()> { Ok(()) }
                fn name(&self) -> &str { "none" }
            }
            Ok(Box::new(NoopBackend))
        }
        other => Err(AppError::Config(format!(
            "Unknown display type '{other}'. Expected: stdout, eink, pipe, or none",
        ))),
    }
}
