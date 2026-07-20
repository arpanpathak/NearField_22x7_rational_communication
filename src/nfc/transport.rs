use std::io::{Read, Write};
use std::time::Duration;

use crate::error::{AppError, AppResult};

const ACK: [u8; 6] = [0x00, 0x00, 0xFF, 0x00, 0xFF, 0x00];
const NACK: [u8; 6] = [0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00];
const PREAMBLE: [u8; 3] = [0x00, 0x00, 0xFF];
const TFI_RESPONSE: u8 = 0xD5;

pub struct SerialTransport {
    port: Box<dyn serialport::SerialPort>,
}

impl SerialTransport {
    pub fn open(path: &str, baud: u32) -> AppResult<Self> {
        let port = serialport::new(path, baud)
            .timeout(Duration::from_millis(200))
            .open()
            .map_err(|e| {
                AppError::Serial(serialport::Error::new(
                    serialport::ErrorKind::NoDevice,
                    format!("Cannot open {}: {}", path, e),
                ))
            })?;
        Ok(Self { port })
    }

    pub fn write_all(&mut self, buf: &[u8]) -> AppResult<()> {
        self.port.write_all(buf)
            .map_err(|e| AppError::Pn532Protocol(format!("Write failed: {}", e)))?;
        Ok(())
    }

    pub fn flush(&mut self) -> AppResult<()> {
        self.port.flush()
            .map_err(|e| AppError::Pn532Protocol(format!("Flush failed: {}", e)))?;
        Ok(())
    }

    pub fn read_exact_ack(&mut self, buf: &mut [u8]) -> AppResult<()> {
        let n = 6;
        let mut offset = 0;
        while offset < n {
            let chunk = &mut buf[..n - offset];
            match self.port.read(chunk) {
                Ok(0) => {
                    return Err(AppError::Pn532Protocol(
                        format!("Read timeout after {}/{} bytes", offset, n),
                    ));
                }
                Ok(bytes_read) => {
                    offset += bytes_read;
                }
                Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {
                    continue;
                }
                Err(e) => {
                    return Err(AppError::Pn532Protocol(format!("Read error: {}", e)));
                }
            }
        }

        let ack = &buf[..6];
        if ack == NACK {
            return Err(AppError::Pn532Protocol("PN532 sent NAK".into()));
        }
        if ack != ACK {
            log::warn!("Expected ACK, got: {:02x?}", ack);
        }
        Ok(())
    }

    pub fn read_response_frame(&mut self, buf: &mut [u8]) -> AppResult<Vec<u8>> {
        self.read_exact_n(buf, 3)?;
        if buf[..3] != PREAMBLE {
            return Err(AppError::Pn532Protocol(
                format!("Bad preamble: {:02x?}", &buf[..3]),
            ));
        }

        self.read_exact_n(buf, 2)?;
        let len = buf[0] as usize;
        let lcs = buf[1];
        if (len as u16 + lcs as u16) & 0xFF != 0 {
            return Err(AppError::Pn532Protocol(
                format!("Length checksum mismatch: LEN={}, LCS={}", len, lcs),
            ));
        }

        let payload_len = len + 2;
        self.read_exact_n(buf, payload_len)?;
        let tail = &buf[..payload_len];

        if tail.is_empty() {
            return Err(AppError::Pn532Protocol("Empty response frame".into()));
        }

        let tfi = tail[0];
        let dcs = tail[tail.len() - 2];

        let data_sum: u16 = tail[..tail.len() - 2].iter().map(|&b| b as u16).sum();
        if ((data_sum + dcs as u16) & 0xFF) != 0 {
            return Err(AppError::Pn532Protocol(
                format!("Data checksum mismatch: sum={:#04x}, DCS={:#04x}", data_sum, dcs),
            ));
        }

        if tfi != TFI_RESPONSE {
            return Err(AppError::Pn532Protocol(
                format!("Unexpected TFI: expected {:#04x}, got {:#04x}", TFI_RESPONSE, tfi),
            ));
        }

        let data = tail[1..tail.len() - 2].to_vec();
        log::trace!("RX <<< {:02x?}", data);
        Ok(data)
    }

    fn read_exact_n(&mut self, buf: &mut [u8], n: usize) -> AppResult<()> {
        let mut offset = 0;
        while offset < n {
            let chunk = &mut buf[..n - offset];
            match self.port.read(chunk) {
                Ok(0) => {
                    return Err(AppError::Pn532Protocol(
                        format!("Read timeout after {}/{} bytes", offset, n),
                    ));
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
