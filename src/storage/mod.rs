//! # Storage Module
//!
//! Persists NFC tag readings to a **local SQLite database**.
//!
//! ## Why SQLite?
//!
//! - **Zero-config** — single file, no server, no daemon.
//! - **Reliable** — ACID transactions, crash-safe (WAL mode).
//! - **Queryable** — can `SELECT` by date, UID, or tag type for later
//!   analysis or display rendering.
//! - **Embedded** — compiles via `rusqlite` bundled feature; no system
//!   SQLite install needed on the Pi.
//!
//! ## Schema
//!
//! ```sql
//! CREATE TABLE nfc_logs (
//!     id          INTEGER PRIMARY KEY AUTOINCREMENT,
//!     uid         TEXT    NOT NULL,
//!     uid_raw     BLOB   NOT NULL,
//!     atqa        BLOB,
//!     sak         INTEGER,
//!     tag_type    TEXT,
//!     timestamp   TEXT    NOT NULL,  -- ISO 8601
//!     created_at  TEXT    NOT NULL DEFAULT (datetime('now'))
//! );
//! ```
//!
//! ## Future Extensions
//!
//! - Add a `counter` column to track how many times each UID has been seen.
//! - Add a `notes` column for manual annotations.
//! - Export to CSV with `SELECT ... INTO OUTFILE` equivalent via application.

pub mod db;
pub mod models;

pub use db::Database;
pub use models::NfcLogEntry;
