use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    pub uid: String,
    pub uid_raw: Vec<u8>,
    pub atqa: Vec<u8>,
    pub sak: u8,
    pub tag_type: String,
    pub timestamp: DateTime<Utc>,
}

impl TagInfo {
    pub fn classify_tag_type(atqa: &[u8], sak: u8) -> String {
        if atqa.len() < 2 {
            return "Unknown (short ATQA)".into();
        }
        let (hi, lo) = (atqa[1], atqa[0]);
        match (hi, lo, sak) {
            (0x00, 0x04, 0x08) => "Mifare Classic 1K".into(),
            (0x00, 0x04, 0x09) => "Mifare Classic Mini".into(),
            (0x00, 0x04, 0x18) | (0x00, 0x02, 0x18) => "Mifare Classic 4K".into(),
            (0x00, 0x44, 0x00) => "Mifare Ultralight".into(),
            (0x00, 0x44, 0x20) => "NTAG21x".into(),
            (0x00, 0x03, _) => "Mifare DESFire".into(),
            (_, _, sak) if (sak & 0x20) != 0 => "ISO/IEC 14443-4 Compliant".into(),
            _ => format!("Unknown Type-A (ATQA={:02x}{:02x}, SAK={:02x})", hi, lo, sak),
        }
    }
}

impl fmt::Display for TagInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] UID={} Type={}",
            self.timestamp.format("%H:%M:%S%.3f"),
            self.uid,
            self.tag_type,
        )
    }
}
