/// NFC tag reading via the PN532 chipset over UART.
///
/// # Architecture
/// This module is decomposed into four sub-modules:
///
/// | Module | Responsibility |
/// |--------|---------------|
/// | [`commands`] | PN532 command codes and parameter payloads |
/// | [`frame`] | UART frame encoding (preamble, checksums) |
/// | [`transport`] | Serial port I/O, ACK/NACK, response parsing |
/// | [`tag`] | TagInfo data model and type classification |
///
/// The [`Pn532`] struct in this module ties them together, providing
/// a high-level API: `open`, `sam_config`, `get_firmware_version`, `poll`.

pub mod commands;
pub mod frame;
pub mod tag;
pub mod transport;

pub use tag::TagInfo;

use std::time::{Duration, Instant};

use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::nfc::commands::Command;
use crate::nfc::transport::SerialTransport;

/// Minimum time (in seconds) between two detections of the same tag.
/// Prevents duplicate logs while a tag is held against the reader.
const DEBOUNCE_SECONDS: u64 = 1;

/// Size of the internal read buffer for serial port operations.
const READ_BUF_SIZE: usize = 512;

/// High-level PN532 driver.
///
/// # Usage
/// ```ignore
/// let mut pn532 = Pn532::open("/dev/ttyAMA0", 115_200)?;
/// pn532.sam_config()?;
/// loop {
///     match pn532.poll() {
///         Ok(tag) => println!("{tag}"),
///         Err(AppError::NoTag) => {}  // no tag in field
///         Err(e) => eprintln!("Error: {e}"),
///     }
///     std::thread::sleep(Duration::from_millis(500));
/// }
/// ```
pub struct Pn532 {
    transport: SerialTransport,
    read_buf: Vec<u8>,
    last_tag_seen: Option<Instant>,
}

impl Pn532 {
    /// Open a serial connection to the PN532.
    ///
    /// `port_path` is typically `/dev/ttyAMA0` (or `/dev/serial0` on Pi OS).
    pub fn open(port_path: &str, baud_rate: u32) -> AppResult<Self> {
        let transport = SerialTransport::open(port_path, baud_rate)?;
        log::info!("Serial port {} opened at {} baud", port_path, baud_rate);
        Ok(Self {
            transport,
            read_buf: vec![0u8; READ_BUF_SIZE],
            last_tag_seen: None,
        })
    }

    /// Query the PN532 firmware version.
    ///
    /// Returns a human-readable string like `"PN532 v1.6"`.
    pub fn get_firmware_version(&mut self) -> AppResult<String> {
        let data = self.send(Command::GetFirmwareVersion)?;
        if data.len() < 4 {
            return Err(AppError::Pn532Protocol(format!(
                "Short firmware response: {} bytes",
                data.len()
            )));
        }
        let ic_name = match data[0] {
            0x32 => "PN532",
            _ => "Unknown PN5xx",
        };
        Ok(format!("{} v{}.{}", ic_name, data[1], data[2]))
    }

    /// Configure the PN532's Security Access Module for tag reading.
    ///
    /// Must be called once after power-on. Sets normal mode with 1-second
    /// virtual card timeout and IRQ enabled.
    pub fn sam_config(&mut self) -> AppResult<()> {
        const SAM_MODE_NORMAL: u8 = 0x01;
        const SAM_TIMEOUT: u8 = 0x14; // 20 * 50ms = 1 second
        const SAM_IRQ_ENABLE: u8 = 0x01;
        const SAM_SUCCESS: u8 = 0x15;

        let data = self.send(Command::SamConfiguration([SAM_MODE_NORMAL, SAM_TIMEOUT, SAM_IRQ_ENABLE]))?;
        if data.is_empty() || data[0] != SAM_SUCCESS {
            return Err(AppError::Pn532Protocol(format!(
                "SAM config unexpected response: {:02x?}",
                data
            )));
        }
        log::info!("PN532 SAM configured (normal mode)");
        Ok(())
    }

    /// Poll for a single NFC tag in the RF field.
    ///
    /// Returns `Ok(TagInfo)` if a tag is detected, or `Err(AppError::NoTag)`
    /// if the field is empty. Implements 1-second debounce to prevent
    /// duplicate log entries.
    pub fn poll(&mut self) -> AppResult<TagInfo> {
        const MAX_TARGETS: u8 = 0x01;
        const BAUD_106KBPS: u8 = 0x00;

        let data = self.send(Command::InListPassiveTarget([MAX_TARGETS, BAUD_106KBPS]))?;

        // Response: [NbTg(1), Tg(1), SENS_RES(2), SEL_RES(1), NFCID1_len(1), NFCID1(N)]
        if data.len() < 3 || data[0] == 0 {
            return Err(AppError::NoTag);
        }

        let sens_res = &data[2..4];
        let sel_res = data[4];
        let uid_len = data[5] as usize;

        if data.len() < 6 + uid_len {
            return Err(AppError::Pn532Protocol(format!(
                "Truncated response: {} bytes, expected {}",
                data.len(),
                6 + uid_len
            )));
        }

        let uid_raw = data[6..6 + uid_len].to_vec();
        let uid_hex: String = uid_raw.iter().map(|b| format!("{:02x}", b)).collect();

        // Debounce: ignore re-reads within DEBOUNCE_SECONDS
        let now = Instant::now();
        if let Some(last) = self.last_tag_seen {
            if now.duration_since(last) < Duration::from_secs(DEBOUNCE_SECONDS) {
                return Err(AppError::NoTag);
            }
        }
        self.last_tag_seen = Some(now);

        Ok(TagInfo {
            uid: uid_hex,
            uid_raw,
            atqa: sens_res.to_vec(),
            sak: sel_res,
            tag_type: TagInfo::classify_tag_type(sens_res, sel_res),
            timestamp: Utc::now(),
        })
    }

    /// Encode a command, send it, read ACK, and parse the response.
    fn send(&mut self, cmd: Command) -> AppResult<Vec<u8>> {
        let packet = frame::encode(cmd);
        log::trace!("TX >>> {:02x?}", packet);

        self.transport.write_all(&packet)?;
        self.transport.flush()?;
        self.transport.read_exact_ack(&mut self.read_buf)?;
        self.transport.read_response_frame(&mut self.read_buf)
    }
}
