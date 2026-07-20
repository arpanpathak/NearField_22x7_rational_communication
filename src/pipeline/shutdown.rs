use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::error::{AppError, AppResult};

pub fn setup_signal_handler() -> AppResult<Arc<AtomicBool>> {
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        log::info!("Received Ctrl-C, shutting down...");
        r.store(false, Ordering::SeqCst);
    })
    .map_err(|e| AppError::Other(format!("Cannot set Ctrl-C handler: {}", e)))?;
    Ok(running)
}
