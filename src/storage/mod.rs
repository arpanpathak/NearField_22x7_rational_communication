/// Persistence layer for NFC tag readings.
///
/// Uses SQLite for reliable, zero-configuration storage. Every tap
/// is persisted with UID, tag type, timestamp, and raw ATQA/SAK bytes.
///
/// See [`db::Database`] for operations and [`models::NfcLogEntry`] for
/// the data model.

pub mod db;
pub mod models;

pub use db::Database;
pub use models::NfcLogEntry;
