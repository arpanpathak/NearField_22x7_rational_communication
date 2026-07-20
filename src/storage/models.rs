use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::nfc::TagInfo;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NfcLogEntry {
    pub id: Option<i64>,
    pub uid: String,
    pub uid_raw: Vec<u8>,
    pub atqa: Option<Vec<u8>>,
    pub sak: Option<u8>,
    pub tag_type: Option<String>,
    pub timestamp: DateTime<Utc>,
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
