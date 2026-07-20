/// Low-level serial transport for the PN532 UART protocol.
///
/// Provides reliable byte-level I/O over a serial port with:
/// - Configurable timeout for non-blocking reads
/// - ACK/NACK verification after every command
/// - Full response frame parsing with checksum validation
///
/// # Protocol constants
/// - ACK (6 bytes): `00 00 FF 00 FF 00` — command accepted
/// - NACK (6 bytes): `00 00 FF FF FF 00` — command rejected
/// - Response TFI: `0xD5` — chip to host

use std::io::{Read, Write};
use std::time::Duration;

use crate::error::{AppError, AppResult};

/// Serial read timeout in milliseconds.
const SERIAL_TIMEOUT_MS: u64 = 200;

/// Size of PN532 ACK/NACK frame in bytes.
const ACK_FRAME_SIZE: usize = 6;

/// Size of frame preamble in bytes.
const PREAMBLE_SIZE: usize = 3;

/// Size of length header (LEN + LCS) in bytes.
const HEADER_SIZE: usize = 2;

/// PN532 ACK pattern: command accepted.
const ACK: [u8; ACK_FRAME_SIZE] = [0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00];

/// PN532 NACK pattern: command rejected.
const NACK: [u8; ACK_FRAME_SIZE] = [0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00];

/// Frame preamble bytes.
const PREAMBLE: [u8; PREAMBLE_SIZE] = [0x00, 0x00, 0xFF];

/// Transport format identifier for chip-to-host frames.
const TFI_RESPONSE: u8 = 0xD5;


/// Wraps a serial port connection to the PN532 chip.
pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl SerialTransport {
    /// Open a serial connection to the PN532.
    ///
    /// `path` is typically `/dev/ttyAMA0` or `/dev/serial0` on a Pi.
    /// `baud` should be 115200 for PN532 HSU mode.
    pub fn open(path: &str, baud: u32) -> AppResult<Self> {
        let port = serialport::new(path, baud)
            .timeout(Duration::from_millis(SERIAL_TIMEOUT_MS))
            .open()
            .map_err(|e| {
                AppError::Serial(serialport::Error::new(
                    serialport::ErrorKind::NoDevice,
                    format!("Cannot open {}: {}", path, e),
                ))
            })?;
        Ok(Self { port })
    }

    /// Write raw bytes to the serial port.
    pub fn write_all(&mut self, buf: &[u8]) -> AppResult<()> {
        self.port
            .write_all(buf)
            .map_err(|e| AppError::Pn532Protocol(format!("Write failed: {}", e)))?;
        Ok(())
    }

    /// Flush the serial port write buffer.
    pub fn flush(&mut self) -> AppResult<()> {
        self.port
            .flush()
            .map_err(|e| AppError::Pn532Protocol(format!("Flush failed: {}", e)))?;
        Ok(())
    }

    /// Read and verify the PN532 ACK frame after a command.
    ///
    /// Returns `Ok(())` if ACK is received. Returns `Err` on NACK or timeout.
    pub fn read_exact_ack(&mut self, buf: &mut [u8]) -> AppResult<()> {
        self.read_exact_n(buf, ACK_FRAME_SIZE)?;

        let ack = &buf[..ACK_FRAME_SIZE];
        if ack == NACK {
            return Err(AppError::Pn532Protocol("PN532 sent NAK".into()));
        }
        if ack != ACK {
            log::warn!("Expected ACK, got: {:02x?}", ack);
        }
        Ok(())
    }

    /// Read and parse a PN532 response frame.
    ///
    /// Validates preamble, length checksum (LCS), TFI, and data checksum (DCS).
    /// Returns the payload data bytes (PDs) from the response.
    pub fn read_response_frame(&mut self, buf: &mut [u8]) -> AppResult<Vec<u8>> {
        // Read and verify preamble (3 bytes: 0x00 0x00 0xFF)
        self.read_exact_n(buf, PREAMBLE_SIZE)?;
        if buf[..PREAMBLE_SIZE] != PREAMBLE {
            return Err(AppError::Pn532Protocol(format!(
                "Bad preamble: {:02x?}",
                &buf[..PREAMBLE_SIZE]
            )));
        }

        // Read length byte and its checksum
        self.read_exact_n(buf, HEADER_SIZE)?;
        let len = buf[0] as usize;
        let lcs = buf[1];
        if (len as u16 + lcs as u16) & 0xFF != 0 {
            return Err(AppError::Pn532Protocol(format!(
                "Length checksum mismatch: LEN={}, LCS={}",
                len, lcs
            )));
        }

        // Read payload (TFI + PDs) + DCS + postamble = len + 2 bytes
        let frame_tail_len = len + 2;
        self.read_exact_n(buf, frame_tail_len)?;
        let tail = &buf[..frame_tail_len];

        if tail.is_empty() {
            return Err(AppError::Pn532Protocol("Empty response frame".into()));
        }

        let tfi = tail[0];
        let dcs = tail[tail.len() - 2];

        // Validate data checksum over (TFI + PDs)
        let data_sum: u16 = tail[..tail.len() - 2].iter().map(|&b| b as u16).sum();
        if ((data_sum + dcs as u16) & 0xFF) != 0 {
            return Err(AppError::Pn532Protocol(format!(
                "Data checksum mismatch: sum={:#04x}, DCS={:#04x}",
                data_sum, dcs
            )));
        }

        // Verify the response is from chip to host
        if tfi != TFI_RESPONSE {
            return Err(AppError::Pn532Protocol(format!(
                "Unexpected TFI: expected {:#04x}, got {:#04x}",
                TFI_RESPONSE, tfi
            )));
        }

        // Extract payload data: bytes between TFI and DCS
        let data = tail[1..tail.len() - 2].to_vec();
        log::trace!("RX <<< {:02x?}", data);
        Ok(data)
    }

    /// Read exactly `n` bytes from the serial port into `buf`.
    ///
    /// Retries on timeout. Returns error on connection loss.
    fn read_exact_n(&mut self, buf: &mut [u8], n: usize) -> AppResult<()> {
        let mut offset = 0;
        while offset < n {
            let chunk = &mut buf[..n - offset];
            match self.port.read(chunk) {
                Ok(0) => {
                    return Err(AppError::Pn532Protocol(format!(
                        "Read timeout after {}/{} bytes",
                        offset, n
                    )));
                }
                Ok(bytes_read) => {
                    offset += bytes_read;
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => {
                    return Err(AppError::Pn532Protocol(format!("Read error: {}", e)));
                }
            }
        }
        Ok(())
    }
}
