/// Output rendering for NFC tag data.
///
/// Provides a trait [`DisplayBackend`] for rendering 22x7 ASCII art frames
/// to various output targets, and a factory function [`backend_from_config`]
/// to select the active backend at runtime.
///
/// # Backends
/// | Backend | Type string | Description |
/// |---------|-------------|-------------|
/// | [`stdout::StdoutBackend`] | `"stdout"` | Terminal output with ANSI clear |
/// | [`pipe::PipeBackend`] | `"pipe"` | JSON frames via named FIFO |
/// | [`eink::EinkBackend`] | `"eink"` | SPI e-ink (stub) |
/// | [`noop::NoopBackend`] | `"none"` | No-op / disabled |

pub mod ascii;
pub mod eink;
pub mod noop;
pub mod pipe;
pub mod stdout;

pub use ascii::AsciiRenderer;

use std::path::PathBuf;
use crate::error::{AppError, AppResult};

/// Trait for rendering a 22x7 ASCII frame to a physical or virtual display.
///
/// Implementations must handle exactly 7 strings, each 22 characters wide.
pub trait DisplayBackend {
    /// Initialise the display device.
    fn init(&mut self) -> AppResult<()>;

    /// Render a pre-formatted 22x7 frame to the display.
    fn display_frame(&mut self, frame: &[String]) -> AppResult<()>;

    /// Clear the display.
    fn clear(&mut self) -> AppResult<()>;
}

/// Construct a display backend from a configuration string.
///
/// # Errors
/// Returns `AppError::Config` if `display_type` is not one of the
/// recognised values: `"stdout"`, `"eink"`, `"pipe"`, `"none"`.
pub fn backend_from_config(
    display_type: &str,
    pipe_path: Option<PathBuf>,
) -> AppResult<Box<dyn DisplayBackend>> {
    match display_type {
        "stdout" => Ok(Box::new(stdout::StdoutBackend::new())),
        "eink" => Ok(Box::new(eink::EinkBackend::new())),
        "pipe" => {
            let path = pipe_path.ok_or_else(|| {
                AppError::Config("pipe backend requires display_pipe path".into())
            })?;
            Ok(Box::new(pipe::PipeBackend::new(path)))
        }
        "none" => Ok(Box::new(noop::NoopBackend)),
        other => Err(AppError::Config(format!(
            "Unknown display type '{}'. Expected: stdout, eink, pipe, or none",
            other
        ))),
    }
}
