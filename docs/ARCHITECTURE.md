# Architecture Guide

> Deep dive into the code design, module responsibilities, data flow,
> and how to extend the project.

---

## 1. High-Level Data Flow

```
┌─────────────────────────────────────────────────────────────────────┐
│                         NEARFIELD SYSTEM                             │
│                                                                     │
│  ┌──────────┐   UART    ┌──────────┐   TagInfo   ┌──────────┐     │
│  │  PN532   │◀────────▶│  nfc/    │ ──────────▶ │ storage/ │     │
│  │ (hardware)│          │ pn532.rs │             │  db.rs   │     │
│  └──────────┘           │  tag.rs  │             │ (SQLite) │     │
│                         └──────────┘             └──────────┘     │
│                               │                                       │
│                               │ TagInfo                               │
│                               ▼                                       │
│                         ┌──────────┐             ┌──────────┐     │
│                         │ display/ │ ──────────▶ │ Backend  │     │
│                         │ ascii.rs │             │ (stdout, │     │
│                         │          │             │  pipe,   │     │
│                         │ 22×7     │             │  eink)   │     │
│                         │ renderer │             └──────────┘     │
│                         └──────────┘                               │
│                                                                     │
│  ┌──────────┐   ┌──────────┐   ┌──────────┐                       │
│  │ config/  │──▶│  main.rs │──▶│ error.rs │  (shared by all)      │
│  │ config.rs│   │ (loop)   │   │ AppError │                       │
│  └──────────┘   └──────────┘   └──────────┘                       │
└─────────────────────────────────────────────────────────────────────┘
```

---

## 2. Module Map

### `src/main.rs` — Entry Point & Event Loop

Responsible for:

1. **Initialisation** (in order):
   - Logging (`env_logger`)
   - Config load (`Config::load()`)
   - Display backend initialisation (`backend_from_config`)
   - Database open + migration (`Database::open`)
   - PN532 initialisation + firmware check + SAM config

2. **Main loop**:
   - Calls `pn532.poll()` every `poll_interval_ms`
   - On tag detection: logs to DB, renders to display, shows for 2 seconds
   - On `NoTag`: sleeps and retries
   - On error: logs, sleeps, re-initialises PN532

3. **Shutdown**:
   - Ctrl-C handler sets `running = false`
   - Loop exits, display cleared, summary printed

### `src/config.rs` — Configuration

- Deserialises `nearfield.toml` (or env-var overrides)
- All fields have sensible defaults
- See [Configuration Reference](#6-configuration-reference) below

### `src/error.rs` — Unified Errors

- `AppError` enum with `thiserror` derive
- `AppResult<T>` type alias for `Result<T, AppError>`
- Categories: Serial, PN532Protocol, NoTag, Database, Display, Config, Io, Other
- `From` impls for `String`, `&str`, and the wrapped library error types

### `src/nfc/` — NFC Tag Reading

**`pn532.rs`** — Low-level PN532 UART frame protocol:
- Implements PN532 frame encoding/decoding per spec Section 7.1.1
- Commands: `GetFirmwareVersion`, `SAMConfiguration`, `InListPassiveTarget`
- Handles ACK/NAK, checksum verification, frame reassembly
- Debounces tags (1-second window)

**`tag.rs`** — Tag data model:
- `TagInfo` struct: uid, uid_raw, atqa, sak, tag_type, timestamp
- `classify_tag_type()`: maps ATQA+SAK to human-readable names
- `Display` impl for logging

### `src/storage/` — Persistence

**`db.rs`** — SQLite operations:
- Auto-creates schema with indexes
- WAL journal mode for concurrent reads
- `insert_tag()`, `recent_entries()`, `total_count()`, `trim_to()`
- Prepared statements via `prepare_cached`

**`models.rs`** — Data models:
- `NfcLogEntry` with optional fields for DB rows
- `From<TagInfo>` conversion

### `src/display/` — Output Rendering

**`ascii.rs`** — 22×7 ASCII art renderer:
- Fixed-width grid: 22 cols × 7 rows
- `render_tag()`: shows UID, type, tap count in a bordered box
- `render_idle()`: "scanning..." placeholder
- `render_history()`: shows recent taps

**`backend.rs`** — Display backends:
- `DisplayBackend` trait: `init()`, `display_frame()`, `clear()`, `name()`
- `StdoutBackend`: prints to terminal (ANSI clear)
- `PipeBackend`: writes JSON to named FIFO
- `EinkBackend`: stub for SPI e-ink hardware

---

## 3. Key Design Decisions

### Why synchronous, not async?

- Polling 2×/second doesn't benefit from async
- Simpler code: no futures, executors, or tokio dependency
- Lower binary size (~3 MB release vs ~8 MB with tokio)
- Easier to debug (linear stack traces)

### Why SQLite, not CSV?

- CSV has no schema, no type safety, no indexes
- SQLite lets us query by UID, date range, tag type
- SQLite is ACID — no corruption on sudden power loss
- CSV export is trivially done via `.mode csv` in sqlite3
- JSON export is built into `Database::export_json()`

### Why 22×7 ASCII, not graphics?

- No font rendering library needed
- 154 bytes per frame (vs ~60 KB for a 250×122 bitmap)
- Works on any display (e-ink, OLED, terminal, web)
- ASCII box-drawing characters look retro-cool

### Why the PipeBackend pattern?

The e-ink SPI driver ecosystem is fragmented (many display models,
different `embedded-hal` implementations). Instead of baking all of
them into the Rust binary, we write frames to a FIFO. An external
script (Python, C, whatever) reads the pipe and handles the
display-specific SPI protocol. This keeps the Rust code clean and
display-agnostic.

---

## 4. Extending the Project

### Add a new tag type

Edit `tag.rs` → `classify_tag_type()`:

```rust
// In nfc/tag.rs, add to the match:
(0x00, 0x04, 0x18) => "Mifare Classic 4K".into(),
```

### Add a new display backend

1. Create a struct implementing `DisplayBackend` in `backend.rs`.
2. Add it to the match in `backend_from_config()`.
3. Optionally add a config field in `config.rs`.

### Read NDEF messages (beyond UID-only)

The current implementation only reads the tag identifier (UID).
To read NDEF records (text, URLs, vCards):

1. After `InListPassiveTarget`, send `InDataExchange` with
   `READ BINARY` commands to the tag.
2. Parse the NDEF TLV structure from the response.
3. Add NDEF fields to `TagInfo`.

This requires ISO 14443-4 compliance (transparent exchange).

### Add a web dashboard

- Use the SQLite database from a separate web server.
- Embed `rusqlite` + `actix-web` to serve a REST API.
- The database path can be shared or the DB copied periodically.

---

## 5. PN532 Protocol Reference

### Frame format

```
Host → PN532:
  00 00 FF LEN LCS D4 CMD PD0..PDn DCS 00

PN532 → Host:
  00 00 FF LEN LCS D5 CMD PD0..PDn DCS 00
```

### Key commands

| Byte  | Command                  | Payload      | Response                  |
|-------|--------------------------|--------------|---------------------------|
| 0x02  | GetFirmwareVersion       | (none)       | IC, Ver, Rev, Support     |
| 0x14  | SAMConfiguration         | Mode, Timeout, IRQ | 0x15 (success)     |
| 0x4A  | InListPassiveTarget      | MaxTg, BaudRate | NbTg, Tg, SENS_RES, SEL_RES, NFCID1... |

### Error handling

The PN532 returns:
- **ACK** (00 00 FF 00 FF 00) = command accepted
- **NACK** (00 00 FF FF FF 00) = command rejected (bad checksum or busy)

After ACK, the response frame follows with the actual data.

---

## 6. Configuration Reference

### `nearfield.toml`

```toml
# Serial port for PN532 (default: /dev/ttyAMA0)
serial_port = "/dev/ttyAMA0"

# Baud rate (default: 115200)
serial_baud = 115200

# SQLite database path (default: ./nearfield.db)
db_path = "./nearfield.db"

# Poll interval in milliseconds (default: 500)
poll_interval_ms = 500

# Display backend: "stdout", "eink", "pipe", or "none" (default: stdout)
display_type = "stdout"

# Named pipe path (only used if display_type = "pipe")
display_pipe = "/tmp/nearfield.fifo"

# Maximum log entries before trimming (0 = unlimited, default: 10000)
max_log_entries = 10000

# Verbosity: 0 = default, 1 = debug, 2 = trace (default: 0)
verbosity = 0
```

### Environment variable overrides

All config fields can be set via environment variables:

```bash
export NEARFIELD_SERIAL_PORT=/dev/ttyUSB0
export NEARFIELD_DB_PATH=/data/nearfield.db
export NEARFIELD_POLL_MS=200
export NEARFIELD_DISPLAY_TYPE=pipe
export NEARFIELD_DISPLAY_PIPE=/tmp/nf.fifo
export RUST_LOG=debug
```

---

## 7. Dependencies

| Crate         | Version | Purpose                                  |
|---------------|---------|------------------------------------------|
| `serialport`  | 4.3     | Cross-platform serial port I/O           |
| `rusqlite`    | 0.32    | Embedded SQLite with bundled compilation |
| `chrono`      | 0.4     | Timestamps with timezone handling        |
| `serde`       | 1       | Serialisation framework                  |
| `serde_json`  | 1       | JSON serialisation for pipe backend      |
| `log`         | 0.4     | Logging facade                           |
| `env_logger`  | 0.11    | Environment-variable-configured logger   |
| `thiserror`   | 2       | Derive-macro error types                 |
| `toml`        | 0.8     | TOML config file parser                  |
| `ctrlc`       | 3.4     | Ctrl-C signal handler                    |

---

## 8. Build Profile

Release builds are optimised for size (`opt-level = "s"`), with LTO
and stripping — resulting in a ~3 MB binary that fits comfortably on
the Pi's limited storage.
