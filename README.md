# NearField_22x7_rational_communication

> **Low-power NFC tag reader for Raspberry Pi Zero 2WH + PN532.**
> Logs taps to SQLite, renders ASCII art on e-ink (or any display).

```
┌────────────────────┐
│  NFC TAG           │
│                    │
│ UID 7ed59290       │
│ TYPE Mifare Classic│
│ tap #42            │
└────────────────────┘
```

## Features

- **Read NFC tags** — PN532 chipset over UART (supports Mifare Classic, Ultralight, NTAG, DESFire, ISO 14443-4)
- **SQLite logging** — every tap stored with UID, type, timestamp; queryable, exportable to JSON
- **ASCII art display** — 22×7 character grid rendered to stdout, named pipe, or e-ink
- **Low power** — 500 ms poll interval, PN532 sleeps between polls, Pi Zero 2WH sips ~0.8 W
- **Debounced** — 1-second debounce prevents duplicate logs while tag is held to reader
- **Graceful shutdown** — Ctrl-C handler flushes DB and clears display
- **Auto-reconnect** — re-initialises PN532 if serial communication drops
- **Log rotation** — configurable max entries to keep the DB lean

## Hardware Requirements

| Component              | Notes                                   |
|------------------------|-----------------------------------------|
| Raspberry Pi Zero 2WH  | With pre-soldered GPIO header           |
| PN532 NFC Module       | Any variant with UART pins (most do)    |
| 4× female-to-female Dupont wires | VCC, GND, TX, RX               |
| MicroSD card (≥ 8 GB)  | With Raspberry Pi OS Lite (64-bit)      |
| Micro-USB power supply | 2.5 A recommended                      |

## Quick Start

### 1. Set up your Pi

Follow [docs/OS_SETUP.md](docs/OS_SETUP.md) to:
- Flash Raspberry Pi OS Lite
- Enable UART on GPIO 14/15
- Connect via SSH or USB serial
- Install Rust

### 2. Wire the PN532

| PN532 Pin | Pi GPIO  | Pi Header Pin |
|-----------|----------|---------------|
| VCC       | 3.3V     | Pin 1         |
| GND       | GND      | Pin 6         |
| TX        | RX (GP15)| Pin 10        |
| RX        | TX (GP14)| Pin 8         |

See [docs/HARDWARE_SETUP.md](docs/HARDWARE_SETUP.md) for photos and details.

### 3. Build and run

```bash
# Clone the repo on your Pi
git clone https://github.com/arpanpathak/nearfield_22x7_rational_communication.git
cd nearfield_22x7_rational_communication

# Build (takes a few minutes on Pi Zero)
cargo build --release

# Run
RUST_LOG=info ./target/release/nearfield_22x7_rational_communication
```

### 4. Tap a tag

Hold any NFC tag (Mifare card, NTAG sticker, phone in card-emulation mode)
close to the PN532 antenna. You'll see:

```
[14:32:01.234] [INFO] [UID=7ed59290] Type=Mifare Classic 1K
```

And the ASCII display will show the tag info.

## Configuration

Create a `nearfield.toml` in the working directory:

```toml
serial_port = "/dev/ttyAMA0"
serial_baud = 115200
db_path = "./nearfield.db"
poll_interval_ms = 500
display_type = "stdout"
max_log_entries = 10000
```

All fields are optional. See [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) for details.

## Project Structure

```
src/
├── main.rs           # Entry point + polling loop
├── config.rs         # TOML config loader
├── error.rs          # Unified error types
├── nfc/
│   ├── mod.rs        # Module re-exports
│   ├── pn532.rs      # PN532 UART protocol driver
│   └── tag.rs        # TagInfo data model
├── storage/
│   ├── mod.rs        # Module re-exports
│   ├── db.rs         # SQLite operations
│   └── models.rs     # NfcLogEntry data model
└── display/
    ├── mod.rs        # Module + trait definition
    ├── ascii.rs      # 22×7 ASCII art renderer
    └── backend.rs    # Stdout, Pipe, Eink backends
```

## Testing

```bash
# Unit tests (no hardware required)
cargo test

# Integration test (requires PN532 connected)
RUST_LOG=debug cargo run --release
```

See [docs/TESTING.md](docs/TESTING.md) for full test procedures.

## License

MIT — see [LICENSE](LICENSE).

## Why "22x7"?

The display renders NFC tag info in a fixed **22-column by 7-row** ASCII art
grid — optimised for small e-ink displays (e.g. Waveshare 2.13" at 250×122 px
with ~11 px per monospace character).

> *"In the middle of difficulty lies opportunity."* — Albert Einstein
