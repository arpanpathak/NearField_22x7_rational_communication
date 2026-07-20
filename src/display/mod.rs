pub mod ascii;
pub mod eink;
pub mod noop;
pub mod pipe;
pub mod stdout;

pub use ascii::AsciiRenderer;

use std::path::PathBuf;
use crate::error::{AppError, AppResult};

pub trait DisplayBackend {
    fn init(&mut self) -> AppResult<()>;
    fn display_frame(&mut self, frame: &[String]) -> AppResult<()>;
    fn clear(&mut self) -> AppResult<()>;
}

pub fn backend_from_config(
    display_type: &str,
    pipe_path: Option<PathBuf>,
) -> AppResult<Box<dyn DisplayBackend>> {
    match display_type {
        "stdout" => Ok(Box::new(stdout::StdoutBackend::new())),
        "eink" => Ok(Box::new(eink::EinkBackend::new())),
        "pipe" => {
            let path = pipe_path
                .ok_or_else(|| AppError::Config("pipe backend requires display_pipe path".into()))?;
            Ok(Box::new(pipe::PipeBackend::new(path)))
        }
        "none" => Ok(Box::new(noop::NoopBackend)),
        other => Err(AppError::Config(format!(
            "Unknown display type '{}'. Expected: stdout, eink, pipe, or none",
            other
        ))),
    }
}
