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

pub struct Pn532 {
    transport: SerialTransport,
    read_buf: Vec<u8>,
    last_tag_seen: Option<Instant>,
}

impl Pn532 {
    pub fn open(port_path: &str, baud_rate: u32) -> AppResult<Self> {
        let transport = SerialTransport::open(port_path, baud_rate)?;
        log::info!("Serial port {} opened at {} baud", port_path, baud_rate);
        Ok(Self {
            transport,
            read_buf: vec![0u8; 512],
            last_tag_seen: None,
        })
    }

    pub fn get_firmware_version(&mut self) -> AppResult<String> {
        let data = self.send(Command::GetFirmwareVersion)?;
        if data.len() < 4 {
            return Err(AppError::Pn532Protocol(
                format!("Short firmware response: {} bytes", data.len()),
            ));
        }
        let ic_name = match data[0] {
            0x32 => "PN532",
            _ => "Unknown PN5xx",
        };
        Ok(format!("{} v{}.{}", ic_name, data[1], data[2]))
    }

    pub fn sam_config(&mut self) -> AppResult<()> {
        let data = self.send(Command::SamConfiguration([0x01, 0x14, 0x01]))?;
        if data.is_empty() || data[0] != 0x15 {
            return Err(AppError::Pn532Protocol(
                format!("SAM config unexpected response: {:02x?}", data),
            ));
        }
        log::info!("PN532 SAM configured (normal mode)");
        Ok(())
    }

    pub fn poll(&mut self) -> AppResult<TagInfo> {
        let data = self.send(Command::InListPassiveTarget([0x01, 0x00]))?;

        if data.len() < 3 || data[0] == 0 {
            return Err(AppError::NoTag);
        }

        let sens_res = &data[2..4];
        let sel_res = data[4];
        let uid_len = data[5] as usize;

        if data.len() < 6 + uid_len {
            return Err(AppError::Pn532Protocol(
                format!("Truncated response: {} bytes, expected {}", data.len(), 6 + uid_len),
            ));
        }

        let uid_raw = data[6..6 + uid_len].to_vec();
        let uid_hex: String = uid_raw.iter().map(|b| format!("{:02x}", b)).collect();

        let now = Instant::now();
        if let Some(last) = self.last_tag_seen {
            if now.duration_since(last) < Duration::from_secs(1) {
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

    fn send(&mut self, cmd: Command) -> AppResult<Vec<u8>> {
        let packet = frame::encode(cmd);
        log::trace!("TX >>> {:02x?}", packet);

        self.transport.write_all(&packet)?;
        self.transport.flush()?;

        self.transport.read_exact_ack(&mut self.read_buf)?;

        self.transport.read_response_frame(&mut self.read_buf)
    }
}
