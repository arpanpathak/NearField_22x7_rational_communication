//! # Display Module
//!
//! Renders NFC tag data as **ASCII art** on an output device.
//!
//! ## Architecture
//!
//! The [`DisplayBackend`] trait abstracts over output targets:
//!
//! - **`StdoutBackend`** — prints to stdout (for testing / debugging).
//! - **`EinkBackend`** — writes to a Waveshare e-ink display via SPI.
//! - **`PipeBackend`** — writes framed ASCII to a FIFO for external
//!   display processors.
//!
//! The [`AsciiRenderer`] converts tabular and tag data into a fixed-width
//! 22×7 ASCII grid — the "22x7" in the project name.
//!
//! ## Display Canon: 22 columns × 7 rows
//!
//! ```text
//! ┌────────────────────┐
//! │  NFC TAG           │  ← Title row
//! │  UID: 7ed59290     │  ← Tag info
//! │  Type: Mifare 1K   │
//! │  Time: 14:32:01    │
//! │                    │
//! │  [TAG]             │  ← Footer / status
//! └────────────────────┘
//! ```

pub mod ascii;
pub mod backend;

pub use ascii::AsciiRenderer;
pub use backend::{DisplayBackend, backend_from_config};
