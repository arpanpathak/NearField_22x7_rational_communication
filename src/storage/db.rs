use std::path::Path;
use rusqlite::{params, Connection};
use crate::error::{AppError, AppResult};
use crate::nfc::TagInfo;
use crate::storage::models::NfcLogEntry;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(db_path: &Path) -> AppResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nfc_logs (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                uid         TEXT    NOT NULL,
                uid_raw     BLOB   NOT NULL,
                atqa        BLOB,
                sak         INTEGER,
                tag_type    TEXT,
                timestamp   TEXT    NOT NULL,
                created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_nfc_logs_uid ON nfc_logs(uid);
            CREATE INDEX IF NOT EXISTS idx_nfc_logs_ts ON nfc_logs(timestamp);"
        )?;
        log::info!("Database opened at {}", db_path.display());
        Ok(Self { conn })
    }

    pub fn insert_tag(&self, tag: &TagInfo) -> AppResult<i64> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO nfc_logs (uid, uid_raw, atqa, sak, tag_type, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
        )?;
        stmt.execute(params![
            tag.uid,
            tag.uid_raw,
            tag.atqa,
            tag.sak as i64,
            tag.tag_type,
            tag.timestamp.to_rfc3339(),
        ])?;
        let row_id = self.conn.last_insert_rowid();
        log::info!("Logged tag {} (id={})", tag.uid, row_id);
        Ok(row_id)
    }

    pub fn recent_entries(&self, limit: u64) -> AppResult<Vec<NfcLogEntry>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, uid, uid_raw, atqa, sak, tag_type, timestamp, created_at
             FROM nfc_logs ORDER BY id DESC LIMIT ?1"
        )?;
        let rows = stmt.query_map(params![limit as i64], |row| {
            Ok(NfcLogEntry {
                id: Some(row.get(0)?),
                uid: row.get(1)?,
                uid_raw: row.get(2)?,
                atqa: row.get(3)?,
                sak: row.get(4)?,
                tag_type: row.get(5)?,
                timestamp: row.get::<_, String>(6)?.parse().unwrap_or_default(),
                created_at: row.get(7)?,
            })
        })?;
        let mut entries = Vec::with_capacity(limit as usize);
        for row in rows {
            entries.push(row?);
        }
        Ok(entries)
    }

    pub fn total_count(&self) -> AppResult<u64> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM nfc_logs", [], |row| row.get(0)
        )?;
        Ok(count as u64)
    }

    pub fn trim_to(&self, max_entries: u64) -> AppResult<u64> {
        let total = self.total_count()?;
        if total <= max_entries {
            return Ok(0);
        }
        let to_delete = total - max_entries;
        self.conn.execute(
            "DELETE FROM nfc_logs WHERE id IN (
                SELECT id FROM nfc_logs ORDER BY id ASC LIMIT ?1
            )",
            params![to_delete as i64],
        )?;
        let deleted = self.conn.changes() as u64;
        log::info!("Trimmed {} old log entries", deleted);
        Ok(deleted)
    }

    pub fn export_json(&self) -> AppResult<String> {
        let entries = self.recent_entries(u64::MAX)?;
        serde_json::to_string_pretty(&entries)
            .map_err(|e| AppError::Other(format!("JSON serialisation: {}", e)))
    }
}
