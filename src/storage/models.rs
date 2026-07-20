//! # Data Models
//!
//! Structs representing rows in the SQLite database and intermediate formats
//! for serialisation (JSON / CSV).

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::nfc::TagInfo;

/// A single NFC tap logged to the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfcLogEntry {
    /// Auto-incremented primary key.
    pub id: Option<i64>,

    /// Hex-encoded UID (e.g. `"7ed59290"`).
    pub uid: String,

    /// Raw UID bytes (stored as SQLite BLOB).
    pub uid_raw: Vec<u8>,

    /// ATQA bytes (Type-A identification).
    pub atqa: Option<Vec<u8>>,

    /// SAK byte (Type-A selection acknowledge).
    pub sak: Option<u8>,

    /// Human-readable tag type.
    pub tag_type: Option<String>,

    /// Timestamp from the `TagInfo` when the tag was first seen.
    pub timestamp: DateTime<Utc>,

    /// When this row was actually inserted (set by the DB).
    pub created_at: Option<String>,
}

impl From<TagInfo> for NfcLogEntry {
    /// Convert a live `TagInfo` into a storable `NfcLogEntry`.
    fn from(tag: TagInfo) -> Self {
        Self {
            id: None,
            uid: tag.uid,
            uid_raw: tag.uid_raw,
            atqa: Some(tag.atqa),
            sak: Some(tag.sak),
            tag_type: Some(tag.tag_type),
            timestamp: tag.timestamp,
            created_at: None,
        }
    }
}
