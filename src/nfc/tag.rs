//! # TagInfo — Data model for a detected NFC tag
//!
//! When the PN532 detects a tag in its RF field, we extract:
//!
//! - **UID** (unique identifier) — the tag's hardware serial number
//! - **ATQA / SAK** — Type-A identification bytes that tell us the tag family
//! - **Tag type** — human-readable name derived from ATQA+SAK
//! - **Timestamp** — when the tag was first seen
//!
//! This struct is the currency of the entire pipeline:
//! `PN532 → TagInfo → SQLite → Display`

use std::fmt;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Information about a single detected NFC tag.
///
/// Clonable, serialisable, storable.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagInfo {
    /// Unique identifier of the tag (hex-encoded, e.g. `"7ed59290"`).
    pub uid: String,

    /// Raw UID bytes (for direct binary comparison).
    pub uid_raw: Vec<u8>,

    /// ATQA (SENS_RES) — two bytes from the tag's anti-collision response.
    pub atqa: Vec<u8>,

    /// SAK (SEL_RES) — one byte indicating tag protocol capabilities.
    pub sak: u8,

    /// Human-readable tag type (e.g. `"Mifare Classic 1K"`, `"NTAG213"`).
    pub tag_type: String,

    /// Timestamp of first detection in this poll cycle.
    pub timestamp: DateTime<Utc>,
}

impl TagInfo {
    /// Derive a human-readable tag type from ATQA and SAK bytes.
    ///
    /// The lookup is based on the NXP application notes and common
    /// ISO/IEC 14443-3 Type-A tag behaviour.
    pub fn classify_tag_type(atqa: &[u8], sak: u8) -> String {
        if atqa.len() < 2 {
            return "Unknown (short ATQA)".into();
        }
        let atqa_lo = atqa[0];
        let atqa_hi = atqa[1];

        match (atqa_hi, atqa_lo, sak) {
            // ── Mifare Classic ─────────────────────────────────────────
            (0x00, 0x04, 0x08) => "Mifare Classic 1K".into(),
            (0x00, 0x04, 0x09) => "Mifare Classic Mini".into(),
            (0x00, 0x04, 0x18) | (0x00, 0x02, 0x18) => "Mifare Classic 4K".into(),

            // ── Mifare Ultralight / NTAG ───────────────────────────────
            (0x00, 0x44, 0x00) => "Mifare Ultralight / Ultralight C".into(),
            (0x00, 0x44, 0x20) => "NTAG21x".into(),

            // ── Mifare DESFire ─────────────────────────────────────────
            (0x00, 0x03, _) => "Mifare DESFire".into(),
            (0x00, 0x44, _) if sak == 0x20 => "NTAG".into(),

            // ── ISO/IEC 14443-4 compliant (SAK bit 5 set) ──────────────
            (_, _, sak) if (sak & 0x20) != 0 => "ISO/IEC 14443-4 Compliant".into(),

            // ── Default fallback ────────────────────────────────────────
            _ => format!("Unknown Type-A (ATQA={atqa_hi:02x}{atqa_lo:02x}, SAK={sak:02x})"),
        }
    }
}

impl fmt::Display for TagInfo {
    /// Human-readable one-liner for logs.
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
