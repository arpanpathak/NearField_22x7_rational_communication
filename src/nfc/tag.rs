/// NFC tag data model and type classification.
///
/// When the PN532 detects a tag, we extract its UID, ATQA, and SAK bytes.
/// This module provides the [`TagInfo`] struct to hold that data and the
/// [`TagInfo::classify_tag_type`] function to identify the tag family.
///
/// # Tag type detection
/// Classification is based on the ISO/IEC 14443-3 Type-A anti-collision
/// response. ATQA (2 bytes) identifies the tag family, and SAK (1 byte)
/// indicates protocol capabilities. See NXP application notes for details.

use std::fmt;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a single detected NFC tag.
///
/// This is the primary data type flowing through the pipeline:
/// `PN532 -> TagInfo -> SQLite -> Display`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// Hex-encoded UID string (e.g. "7ed59290").
    pub uid: String,
    /// Raw UID bytes for binary comparison.
    pub uid_raw: Vec<u8>,
    /// ATQA (SENS_RES) bytes from the Type-A anti-collision response.
    pub atqa: Vec<u8>,
    /// SAK (SEL_RES) byte indicating protocol capabilities.
    pub sak: u8,
    /// Human-readable tag type (e.g. "Mifare Classic 1K").
    pub tag_type: String,
    /// UTC timestamp of first detection.
    pub timestamp: DateTime<Utc>,
}

impl TagInfo {
    /// Derive a human-readable tag type from ATQA and SAK bytes.
    ///
    /// The lookup table is based on NXP application notes and common
    /// ISO/IEC 14443-3 Type-A tag behaviour.
    ///
    /// # Parameters
    /// - `atqa`: 2-byte SENS_RES from the tag's anti-collision response
    /// - `sak`: 1-byte SEL_RES from the tag's select acknowledge
    pub fn classify_tag_type(atqa: &[u8], sak: u8) -> String {
        if atqa.len() < 2 {
            return "Unknown (short ATQA)".into();
        }
        let (hi, lo) = (atqa[1], atqa[0]);
        match (hi, lo, sak) {
            // Mifare Classic
            (0x00, 0x04, 0x08) => "Mifare Classic 1K".into(),
            (0x00, 0x04, 0x09) => "Mifare Classic Mini".into(),
            (0x00, 0x04, 0x18) | (0x00, 0x02, 0x18) => "Mifare Classic 4K".into(),
            // Mifare Ultralight / NTAG
            (0x00, 0x44, 0x00) => "Mifare Ultralight".into(),
            (0x00, 0x44, 0x20) => "NTAG21x".into(),
            // Mifare DESFire (ISO 14443-4)
            (0x00, 0x03, _) => "Mifare DESFire".into(),
            // Any tag with ISO 14443-4 compliance (SAK bit 5)
            (_, _, sak) if (sak & 0x20) != 0 => "ISO/IEC 14443-4 Compliant".into(),
            // Fallback
            _ => format!("Unknown Type-A (ATQA={:02x}{:02x}, SAK={:02x})", hi, lo, sak),
        }
    }
}

impl fmt::Display for TagInfo {
    /// One-line log format: `[HH:MM:SS.fff] UID=XXXXXXXX Type=Mifare Classic 1K`
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
