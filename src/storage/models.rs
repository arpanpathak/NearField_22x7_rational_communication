/// Data models for the SQLite persistence layer.
///
/// [`NfcLogEntry`] represents a single NFC tap record, mirroring the
/// database schema. It can be constructed from a [`TagInfo`] via the
/// `From` trait implementation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::nfc::TagInfo;

/// A single NFC tap logged to the database.
///
/// All fields except `id` and `created_at` are populated from the
/// [`TagInfo`] at insertion time. `id` and `created_at` are set by
/// the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfcLogEntry {
    /// Auto-incremented primary key (None before insert).
    pub id: Option<i64>,
    /// Hex-encoded UID (e.g. "7ed59290").
    pub uid: String,
    /// Raw UID bytes stored as SQLite BLOB.
    pub uid_raw: Vec<u8>,
    /// ATQA identification bytes.
    pub atqa: Option<Vec<u8>>,
    /// SAK selection acknowledge byte.
    pub sak: Option<u8>,
    /// Human-readable tag type.
    pub tag_type: Option<String>,
    /// Timestamp from the TagInfo (when the tag was first seen).
    pub timestamp: DateTime<Utc>,
    /// Database insert timestamp (ISO 8601, set by SQLite).
    pub created_at: Option<String>,
}

impl From<TagInfo> for NfcLogEntry {
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
