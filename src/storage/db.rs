/// SQLite database operations for NFC log persistence.
///
/// Provides schema migration, CRUD operations, automatic log trimming,
/// and JSON export. Uses WAL mode for crash safety and concurrent reads.
///
/// # Schema
/// ```sql
/// CREATE TABLE nfc_logs (
///     id         INTEGER PRIMARY KEY AUTOINCREMENT,
///     uid        TEXT    NOT NULL,
///     uid_raw    BLOB   NOT NULL,
///     atqa       BLOB,
///     sak        INTEGER,
///     tag_type   TEXT,
///     timestamp  TEXT    NOT NULL,  -- ISO 8601
///     created_at TEXT    NOT NULL DEFAULT (datetime('now'))
/// );
/// ```

use std::path::Path;
use rusqlite::{params, Connection};
use crate::error::{AppError, AppResult};
use crate::nfc::TagInfo;
use crate::storage::models::NfcLogEntry;

// ── SQL Query Constants ────────────────────────────────────────────────

/// Enable WAL journal mode for better concurrent-read performance.
const PRAGMA_WAL: &str = "PRAGMA journal_mode=WAL;";

/// Schema migration: create table and indexes if they don't exist.
const SQL_CREATE_TABLES: &str = "\
CREATE TABLE IF NOT EXISTS nfc_logs (
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
CREATE INDEX IF NOT EXISTS idx_nfc_logs_ts  ON nfc_logs(timestamp);";

/// Insert a new tag reading.
const SQL_INSERT: &str = "\
INSERT INTO nfc_logs (uid, uid_raw, atqa, sak, tag_type, timestamp)
VALUES (?1, ?2, ?3, ?4, ?5, ?6)";

/// Query recent entries in reverse chronological order.
const SQL_RECENT: &str = "\
SELECT id, uid, uid_raw, atqa, sak, tag_type, timestamp, created_at
FROM nfc_logs
ORDER BY id DESC
LIMIT ?1";

/// Count total entries.
const SQL_COUNT: &str = "SELECT COUNT(*) FROM nfc_logs";

/// Delete oldest entries beyond a threshold.
const SQL_TRIM: &str = "\
DELETE FROM nfc_logs WHERE id IN (
    SELECT id FROM nfc_logs ORDER BY id ASC LIMIT ?1
)";

/// Database wrapper for NFC log persistence.
pub struct Database {
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at `db_path` and run schema migrations.
    pub fn open(db_path: &Path) -> AppResult<Self> {
        let conn = Connection::open(db_path)?;
        conn.execute_batch(PRAGMA_WAL)?;
        conn.execute_batch(SQL_CREATE_TABLES)?;
        log::info!("Database opened at {}", db_path.display());
        Ok(Self { conn })
    }

    /// Insert a new tag reading into the log.
    ///
    /// Returns the auto-generated row ID.
    pub fn insert_tag(&self, tag: &TagInfo) -> AppResult<i64> {
        let mut stmt = self.conn.prepare_cached(SQL_INSERT)?;
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

    /// Query the most recent `limit` log entries.
    pub fn recent_entries(&self, limit: u64) -> AppResult<Vec<NfcLogEntry>> {
        let mut stmt = self.conn.prepare_cached(SQL_RECENT)?;
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

    /// Count total entries in the database.
    pub fn total_count(&self) -> AppResult<u64> {
        let count: i64 = self.conn.query_row(SQL_COUNT, [], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// Delete old entries beyond `max_entries`, keeping the most recent ones.
    ///
    /// Returns the number of deleted rows.
    pub fn trim_to(&self, max_entries: u64) -> AppResult<u64> {
        let total = self.total_count()?;
        if total <= max_entries {
            return Ok(0);
        }
        let to_delete = total - max_entries;
        self.conn.execute(SQL_TRIM, params![to_delete as i64])?;
        let deleted = self.conn.changes() as u64;
        log::info!("Trimmed {} old log entries", deleted);
        Ok(deleted)
    }

    /// Export all entries as a formatted JSON array string.
    pub fn export_json(&self) -> AppResult<String> {
        let entries = self.recent_entries(u64::MAX)?;
        serde_json::to_string_pretty(&entries)
            .map_err(|e| AppError::Other(format!("JSON serialisation: {}", e)))
    }
}
