//! # NFC Module
//!
//! Provides a high-level interface for reading NFC tags using a **PN532**
//! chipset over **UART** (serial).
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────┐   UART (serial)   ┌───────────┐    RF Field    ┌────────┐
//! │  main.rs   │ ────────────────→ │  Pn532    │ ────────────→ │ NFC Tag │
//! │  (caller)  │ ←──────────────── │  .poll()  │ ←──────────── │        │
//! └────────────┘                   └───────────┘               └────────┘
//! ```
//!
//! The [`Pn532`] struct opens a serial port, negotiates with the PN532
//! firmware (SAM configuration), and exposes a simple `poll()` method that
//! blocks until a tag enters the RF field, returning a [`TagInfo`].
//!
//! ## PN532 UART Frame Protocol
//!
//! Every command/response uses this packet format:
//!
//! | Byte(s)   | Field        | Description                            |
//! |-----------|--------------|----------------------------------------|
//! | 0x00 0x00 | Preamble     | Start of frame                         |
//! | 0xFF      | Start code   | Always 0xFF                            |
//! | 1 byte    | Length (LEN) | Number of bytes in payload (TFI + PDs) |
//! | 1 byte    | LCS          | Checksum: `0x100 - LEN`                |
//! | 1 byte    | TFI          | 0xD4 (host→PN532) or 0xD5 (PN532→host)|
//! | N bytes   | PD0..PDn     | Payload data                            |
//! | 1 byte    | DCS          | Data checksum: `0x100 - sum(TFI+PDs)`  |
//! | 0x00      | Postamble    | End of frame                           |
//!
//! Reference: *PN532 User Manual Rev. 02 — Section 7.1.1 (UART)*

pub mod pn532;
pub mod tag;

pub use pn532::Pn532;
pub use tag::TagInfo;
