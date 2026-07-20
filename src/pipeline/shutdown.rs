/// Graceful shutdown via OS signal handling.
///
/// Installs a SIGINT (Ctrl-C) handler that flips an `AtomicBool` flag.
/// The main loop checks this flag on each iteration and exits cleanly.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use crate::error::{AppError, AppResult};

/// Install a Ctrl-C handler and return a shared shutdown flag.
///
/// The returned `Arc<AtomicBool>` starts as `true`. When the user presses
/// Ctrl-C, the flag is set to `false`, signalling the main loop to exit.
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
