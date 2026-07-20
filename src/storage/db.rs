//! # Database — SQLite Operations
//!
//! Encapsulates the SQLite connection, schema migration, and CRUD operations
//! for NFC log entries.
//!
//! ## Connection Management
//!
//! - Opens a single connection (we're single-threaded; no pool needed).
//! - Enables **WAL mode** for better concurrent-read performance if we
//!   ever add a web viewer alongside the polling loop.
//! - All writes go through prepared statements for safety and speed.

use std::path::Path;

use rusqlite::{params, Connection};

use crate::error::{AppError, AppResult};
use crate::storage::models::NfcLogEntry;
use crate::nfc::TagInfo;

/// Database wrapper for NFC log persistence.
pub struct Database {
    /// SQLite connection handle.
    conn: Connection,
}

impl Database {
    /// Open (or create) the database at `db_path` and run migrations.
    ///
    /// ## Panics
    /// If the schema migration SQL is invalid (this is a programming error).
    pub fn open(db_path: &Path) -> AppResult<Self> {
        let conn = Connection::open(db_path)?;

        // Enable WAL mode for performance.
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;

        // Create the table if it doesn't exist.
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS nfc_logs (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                uid         TEXT    NOT NULL,
                uid_raw     BLOB   NOT NULL,
                atqa        BLOB,
                sak         INTEGER,
                tag_type    TEXT,
                timestamp   TEXT    NOT NULL,  -- ISO 8601
                created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
            );

            -- Index on UID for fast lookups.
            CREATE INDEX IF NOT EXISTS idx_nfc_logs_uid ON nfc_logs(uid);

            -- Index on timestamp for chronological queries.
            CREATE INDEX IF NOT EXISTS idx_nfc_logs_ts ON nfc_logs(timestamp);
            ",
        )?;

        log::info!("Database opened at {}", db_path.display());
        Ok(Self { conn })
    }

    /// Insert a new NFC tag reading into the log.
    ///
    /// Returns the auto-generated row ID.
    pub fn insert_tag(&self, tag: &TagInfo) -> AppResult<i64> {
        let mut stmt = self.conn.prepare_cached(
            "INSERT INTO nfc_logs (uid, uid_raw, atqa, sak, tag_type, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        )?;

        let timestamp_str = tag.timestamp.to_rfc3339();

        stmt.execute(params![
            tag.uid,
            tag.uid_raw,
            tag.atqa,
            tag.sak as i64,
            tag.tag_type,
            timestamp_str,
        ])?;

        let row_id = self.conn.last_insert_rowid();
        log::info!("Logged tag {uid} (id={row_id})", uid = tag.uid);
        Ok(row_id)
    }

    /// Query the most recent `limit` log entries.
    pub fn recent_entries(&self, limit: u64) -> AppResult<Vec<NfcLogEntry>> {
        let mut stmt = self.conn.prepare_cached(
            "SELECT id, uid, uid_raw, atqa, sak, tag_type, timestamp, created_at
             FROM nfc_logs
             ORDER BY id DESC
             LIMIT ?1",
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

    /// Count total entries in the database.
    pub fn total_count(&self) -> AppResult<u64> {
        let count: i64 = self
            .conn
            .query_row("SELECT COUNT(*) FROM nfc_logs", [], |row| row.get(0))?;
        Ok(count as u64)
    }

    /// Delete old entries beyond `max_entries`, keeping the most recent ones.
    ///
    /// Useful for automatic log rotation to prevent the DB from growing
    /// unbounded on the Pi's limited storage.
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
        log::info!("Trimmed {deleted} old log entries");
        Ok(deleted)
    }

    /// Export all entries as a JSON array string.
    pub fn export_json(&self) -> AppResult<String> {
        let entries = self.recent_entries(u64::MAX)?;
        serde_json::to_string_pretty(&entries)
            .map_err(|e| AppError::Other(format!("JSON serialisation: {e}")))
    }
}
