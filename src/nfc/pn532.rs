//! # PN532 Driver (UART mode)
//!
//! Implements the low-level PN532 frame protocol and exposes a high-level
//! `poll()` -> `TagInfo` interface.
//!
//! ## UART Wiring (Pi Zero 2WH ↔ PN532)
//!
//! | PN532 Pin | Pi GPIO  | Pi Header | Notes                        |
//! |-----------|----------|-----------|------------------------------|
//! | VCC       | 3.3V     | Pin 1     | 3.3V (NOT 5V!)               |
//! | GND       | GND      | Pin 6     | Common ground                |
//! | TX        | RX (GP15)| Pin 10    | PN532 TX → Pi RX             |
//! | RX        | TX (GP14)| Pin 8     | Pi TX → PN532 RX             |
//!
//! **Before using UART on Pi:**
//! 1. Enable UART: `sudo raspi-config` → Interface Options → Serial Port
//!    → "No" to login shell, "Yes" to serial hardware.
//! 2. Should see `/dev/serial0` → symlink to `/dev/ttyAMA0`.
//!
//! ## Frame Encoding (Host → PN532)
//!
//! Every command goes through [`Self::send_frame`]:
//!
//! ```text
//! Preamble(2) + Start(1) + LEN(1) + LCS(1) + TFI(1) + PDs(N) + DCS(1) + Post(1)
//! ```
//!
//! Where:
//! - `LCS = 0x100 - LEN`
//! - `DCS = 0x100 - (TFI + sum(PDs)) & 0xFF`
//!
//! The PN532 echoes the frame, then sends a response with the same structure.
//!
//! ## Key Commands Used
//!
//! | Command             | Code      | Purpose                               |
//! |---------------------|-----------|---------------------------------------|
//! | SAMConfiguration    | 0x14      | Wake up chip, configure normal mode   |
//! | GetFirmwareVersion  | 0x02      | Verify chip is alive and responding   |
//! | InListPassiveTarget | 0x4A      | Poll for tags in the RF field         |
//!
//! Reference: *PN532 User Manual Rev. 02 — Section 7.3*
//!
//! ## Power-Saving Notes
//!
//! - PN532 enters low-power mode between polls automatically.
//! - The poll interval (500 ms default) keeps average power low.
//! - We do NOT use the PN532's hardware low-power wake-up (too complex for
//!   this use case; polling is simpler and more reliable).

use std::io::{Read, Write};
use std::time::{Duration, Instant};

use chrono::Utc;

use crate::error::{AppError, AppResult};
use crate::nfc::tag::TagInfo;

// ── PN532 Command / Response Identifiers ────────────────────────────────

const TFI_HOST_TO_PN532: u8 = 0xD4;
const TFI_PN532_TO_HOST: u8 = 0xD5;

const CMD_SAM_CONFIGURATION: u8 = 0x14;
const CMD_GET_FIRMWARE_VERSION: u8 = 0x02;
const CMD_IN_LIST_PASSIVE_TARGET: u8 = 0x4A;

const PN532_PREAMBLE: [u8; 3] = [0x00, 0x00, 0xFF];
const PN532_ACK: [u8; 6] = [0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00];
const PN532_NACK: [u8; 6] = [0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00];


// ── PN532 Driver ────────────────────────────────────────────────────────

/// Driver for the PN532 NFC chip over UART.
///
/// ## Usage
///
/// ```ignore
/// let mut pn532 = Pn532::open("/dev/ttyAMA0", 115_200)?;
/// pn532.sam_config()?;
/// loop {
///     match pn532.poll() {
///         Ok(tag) => println!("Tag detected: {tag}"),
///         Err(AppError::NoTag) => {} // No tag right now, keep polling
///         Err(e) => eprintln!("Error: {e}"),
///     }
///     std::thread::sleep(Duration::from_millis(500));
/// }
/// ```
pub struct Pn532 {
    /// The underlying serial port connection.
    port: Box<dyn serialport::SerialPort>,
    /// Buffer for receiving raw bytes from the PN532.
    read_buf: Vec<u8>,
    /// Timestamp of the last detected tag (debounce).
    last_tag_seen: Option<Instant>,
}

impl Pn532 {
    /// Open a serial connection to the PN532.
    ///
    /// `port_path` is typically `/dev/ttyAMA0` or `/dev/serial0` on a Pi.
    pub fn open(port_path: &str, baud_rate: u32) -> AppResult<Self> {
        let port = serialport::new(port_path, baud_rate)
            .timeout(Duration::from_millis(200)) // Read timeout
            .open()
            .map_err(|e| {
                AppError::Serial(serialport::Error::new(
                    serialport::ErrorKind::NoDevice,
                    format!("Cannot open {port_path}: {e}"),
                ))
            })?;

        log::info!("Serial port {port_path} opened at {baud_rate} baud");

        Ok(Self {
            port,
            read_buf: vec![0u8; 512],
            last_tag_seen: None,
        })
    }

    // ── High-level API ──────────────────────────────────────────────────

    /// Check the firmware version to verify the PN532 is alive.
    ///
    /// Returns the firmware version string (e.g. `"PN532 v1.6"`) or an error.
    pub fn get_firmware_version(&mut self) -> AppResult<String> {
        let response = self.send_command(CMD_GET_FIRMWARE_VERSION, &[])?;
        // Response: [IC(1), Ver(1), Rev(1), Support(1)]
        if response.len() < 4 {
            return Err(AppError::Pn532Protocol(
                format!("Short firmware response: {} bytes", response.len()),
            ));
        }
        let ic = response[0];
        let ver = response[1];
        let rev = response[2];
        let _support = response[3];

        let ic_name = match ic {
            0x32 => "PN532",
            _ => "Unknown PN5xx",
        };

        Ok(format!("{ic_name} v{ver}.{rev}"))
    }

    /// Configure the PN532's **SAM** (Security Access Module) for normal
    /// read/write operation.
    ///
    /// Must be called once after power-on before any tag operations.
    ///
    /// SAM configuration parameters:
    /// - Mode = 0x01 (Normal mode)
    /// - Timeout = 0x14 (20 × 50 ms = 1 s virtual card timeout)
    /// - IRQ = 0x01 (Enable virtual card IRQ)
    pub fn sam_config(&mut self) -> AppResult<()> {
        let params = [0x01, 0x14, 0x01];
        let response = self.send_command(CMD_SAM_CONFIGURATION, &params)?;
        // Successful SAM config returns a single byte 0x15.
        if response.is_empty() || response[0] != 0x15 {
            return Err(AppError::Pn532Protocol(
                format!("SAM config unexpected response: {response:02x?}"),
            ));
        }
        log::info!("PN532 SAM configured (normal mode)");
        Ok(())
    }

    /// Poll for a single NFC tag in the RF field.
    ///
    /// Returns [`TagInfo`] if a tag is present, or [`AppError::NoTag`] if
    /// the RF field is empty.
    ///
    /// This method uses `InListPassiveTarget` with a single-frame timeout
    /// of 200 ms (set via serial port timeout). If no tag responds within
    /// that window, we return `NoTag`.
    ///
    /// ## Debounce
    /// After detecting a tag, we ignore it for 1 second to prevent duplicate
    /// log entries while the tag is held against the reader.
    pub fn poll(&mut self) -> AppResult<TagInfo> {
        // InListPassiveTarget: max targets = 1, baud rate = 106 kbps (Type A)
        let params = [0x01, 0x00];
        let response = self.send_command(CMD_IN_LIST_PASSIVE_TARGET, &params)?;

        // Response format:
        // [NbTg(1), Tg(1), SENS_RES(2), SEL_RES(1), NFCID1_len(1), NFCID1(N), ...]
        if response.len() < 3 {
            return Err(AppError::NoTag);
        }

        let nbtg = response[0];
        if nbtg == 0 {
            return Err(AppError::NoTag);
        }

        let _tg = response[1];         // Target number (always 1)
        let sens_res = &response[2..4]; // ATQA
        let sel_res = response[4];      // SAK
        let uid_len = response[5] as usize;

        if response.len() < 6 + uid_len {
            return Err(AppError::Pn532Protocol(
                format!(
                    "Truncated InListPassiveTarget response: {} bytes, expected {}",
                    response.len(),
                    6 + uid_len,
                ),
            ));
        }

        let uid_raw = response[6..6 + uid_len].to_vec();

        // Hex-encode the UID for human-readable display: lowercase, no separator.
        let uid_hex: String = uid_raw.iter().map(|b| format!("{b:02x}").to_string()).collect();

        // ── Debounce ────────────────────────────────────────────────────
        let now = Instant::now();
        if let Some(last) = self.last_tag_seen {
            if now.duration_since(last) < Duration::from_secs(1) {
                return Err(AppError::NoTag);
            }
        }
        self.last_tag_seen = Some(now);

        let tag_type = TagInfo::classify_tag_type(sens_res, sel_res);

        Ok(TagInfo {
            uid: uid_hex,
            uid_raw,
            atqa: sens_res.to_vec(),
            sak: sel_res,
            tag_type,
            timestamp: Utc::now(),
        })
    }

    // ── Low-level UART Frame Protocol ───────────────────────────────────

    /// Send a command (by command code + payload) and read back the response.
    ///
    /// Wraps the payload in the PN532 UART frame format and handles ACK/NAK.
    ///
    /// Returns the **payload data bytes (PDs)** from the response frame
    /// (i.e. the bytes after TFI, before DCS).
    fn send_command(&mut self, cmd: u8, payload: &[u8]) -> AppResult<Vec<u8>> {
        // Build the raw packet.
        let mut packet = Vec::with_capacity(7 + payload.len());
        packet.extend_from_slice(&PN532_PREAMBLE);
        let len = 2 + payload.len(); // TFI(1) + CMD(1) + payload(N)
        packet.push(len as u8);
        packet.push((0x100 - len as u16) as u8); // LCS
        packet.push(TFI_HOST_TO_PN532);
        packet.push(cmd);
        packet.extend_from_slice(payload);

        // DCS = 0x100 - sum(TFI, CMD, payload...)
        let dcs_sum: u16 = payload.iter().map(|&b| b as u16).sum::<u16>() + cmd as u16 + TFI_HOST_TO_PN532 as u16;
        packet.push((0x100 - (dcs_sum & 0xFF)) as u8);
        packet.push(0x00); // Postamble

        // ── Send ────────────────────────────────────────────────────────
        log::trace!("TX >>> {:02x?}", packet);
        self.port.write_all(&packet).map_err(|e| {
            AppError::Pn532Protocol(format!("Write failed: {e}"))
        })?;
        self.port.flush().map_err(|e| {
            AppError::Pn532Protocol(format!("Flush failed: {e}"))
        })?;

        // ── Read ACK (or NACK) ─────────────────────────────────────────
        let ack = self.read_exact_bytes(6)?;
        if ack == PN532_NACK {
            // NACK means the PN532 was busy or the checksum was wrong.
            return Err(AppError::Pn532Protocol("PN532 sent NAK".into()));
        }
        if ack != PN532_ACK {
            // Unexpected bytes; try to drain garbage and report.
            log::warn!("Expected ACK, got: {ack:02x?}");
            // Read a full frame anyway in case this is a direct response.
            return self.read_response_frame();
        }

        // ── Read response frame ─────────────────────────────────────────
        self.read_response_frame()
    }

    /// Read exactly `n` bytes, retrying until we have them or the serial
    /// timeout fires.
    fn read_exact_bytes(&mut self, n: usize) -> AppResult<Vec<u8>> {
        let mut buf = Vec::with_capacity(n);
        let mut offset = 0;
        while offset < n {
            let chunk = &mut self.read_buf[..n - offset];
            match self.port.read(chunk) {
                Ok(0) => {
                    // No data available within the timeout.
                    return Err(AppError::Pn532Protocol(
                        format!("Read timeout after {offset}/{n} bytes"),
                    ));
                }
                Ok(bytes_read) => {
                    buf.extend_from_slice(&chunk[..bytes_read]);
                    offset += bytes_read;
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    // Retry on timeout — the PN532 may be slow to respond.
                    continue;
                }
                Err(e) => {
                    return Err(AppError::Pn532Protocol(format!("Read error: {e}")));
                }
            }
        }
        Ok(buf)
    }

    /// Parse a full response frame from the PN532.
    ///
    /// Frame structure: `Preamble(3) LEN(1) LCS(1) TFI(1) PDs(N) DCS(1) Post(1)`
    ///
    /// Returns just the payload data bytes (PDs).
    fn read_response_frame(&mut self) -> AppResult<Vec<u8>> {
        // ── Read preamble (0x00 0x00 0xFF) ─────────────────────────────
        let preamble = self.read_exact_bytes(3)?;
        if preamble != PN532_PREAMBLE {
            // Try to resync: drain bytes until we find 0xFF.
            log::warn!("Bad preamble: {preamble:02x?}, trying to resync...");
            return Err(AppError::Pn532Protocol(
                format!("Bad preamble: {preamble:02x?}"),
            ));
        }

        // ── Read Len + LCS ──────────────────────────────────────────────
        let header = self.read_exact_bytes(2)?;
        let len = header[0];
        let lcs = header[1];

        // Validate length checksum.
        if (len as u16 + lcs as u16) & 0xFF != 0 {
            return Err(AppError::Pn532Protocol(format!(
                "Length checksum mismatch: LEN={len}, LCS={lcs}",
            )));
        }

        // ── Read payload (TFI + PDs) + DCS + Postamble ─────────────────
        let payload_len = len as usize + 2; // +1 for DCS, +1 for Postamble
        let tail = self.read_exact_bytes(payload_len)?;

        if tail.is_empty() {
            return Err(AppError::Pn532Protocol("Empty response frame".into()));
        }

        let tfi = tail[0];
        let dcs = tail[tail.len() - 2];
        let _postamble = tail[tail.len() - 1];

        // Validate data checksum.
        let data_sum: u16 = tail[..tail.len() - 2].iter().map(|&b| b as u16).sum();
        if ((data_sum + dcs as u16) & 0xFF) != 0 {
            return Err(AppError::Pn532Protocol(format!(
                "Data checksum mismatch: sum={data_sum:#04x}, DCS={dcs:#04x}",
            )));
        }

        // Verify TFI indicates a response (PN532→Host).
        if tfi != TFI_PN532_TO_HOST {
            return Err(AppError::Pn532Protocol(format!(
                "Unexpected TFI: expected {TFI_PN532_TO_HOST:#04x}, got {tfi:#04x}",
            )));
        }

        // The response payload data starts after TFI (1 byte) and ends
        // before DCS (2 bytes from end).
        let data = tail[1..tail.len() - 2].to_vec();
        log::trace!("RX <<< {:02x?}", data);

        Ok(data)
    }
}

impl Drop for Pn532 {
    fn drop(&mut self) {
        log::info!("Closing PN532 serial port");
    }
}
