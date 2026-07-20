
<p align="center">
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/pi-zero_2wh-success?logo=raspberrypi" alt="Pi Zero">
  <img src="https://img.shields.io/badge/PN532-UART-blue" alt="PN532">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="MIT">
</p>

<p align="center">
  <code>22/7 ≈ π</code>
</p>

<h1 align="center">📡 NearField_22x7_rational_communication</h1>

<p align="center">
  <b>Low-power NFC tag reader</b> for <b>Raspberry Pi Zero 2WH</b> + <b>PN532</b>
  <br>
  🗄️ Logs taps to SQLite &nbsp;|&nbsp; 🎨 Renders ASCII art on e-ink
</p>

<br>

<div align="center">

```
┌────────────────────┐
│  📡 NFC TAG        │
│                    │
│ 🔑 UID 7ed59290   │
│ 🏷️ TYPE Mifare 1K │
│ 📍 tap #42         │
└────────────────────┘
```

</div>

<br>

> **🤔 Why the name?** `22/7` is a classic rational approximation of **π** (pi). This project runs on a **Pi** Zero, reads **NFC** tags, and is all about **rational communication** between devices. The display grid just happens to be 22 columns × 7 rows. Serendipity. ✨

---

## ✨ Features

| 🌟 Feature | 💡 What it does |
|---|---|
| 📡 **Read NFC tags** | PN532 chipset over UART. Supports Mifare Classic, Ultralight, NTAG, DESFire, ISO 14443-4 |
| 🗄️ **SQLite logging** | Every tap stored with UID, type, timestamp. Queryable. Exportable to JSON |
| 🎨 **ASCII art display** | 22×7 character grid → stdout, named pipe, or e-ink |
| 🔋 **Low power** | 500 ms poll interval. PN532 sleeps between polls. Pi Zero sips ~0.8 W |
| 🛡️ **Debounced** | 1-second window prevents duplicate logs while tag is held |
| 🚦 **Graceful shutdown** | Ctrl-C handler flushes DB + clears display |
| 🔄 **Auto-reconnect** | Re-initialises PN532 if serial drops |
| 🗑️ **Log rotation** | Configurable max entries keeps DB lean |

---

## 🧰 Hardware Requirements

| Component | Notes |
|---|---|
| 🥧 **Raspberry Pi Zero 2WH** | With pre-soldered GPIO header |
| 🛸 **PN532 NFC Module** | Any variant with UART pins |
| 🔌 **4× Dupont wires** (F/F) | VCC, GND, TX, RX |
| 💾 **MicroSD card** (≥ 8 GB) | Raspberry Pi OS Lite (64-bit) |
| ⚡ **Micro-USB power supply** | 2.5 A recommended |

---

## 🚀 Quick Start

### 🔹 1. Set up your Pi

Follow [`docs/OS_SETUP.md`](docs/OS_SETUP.md) for:
- 💿 Flashing Raspberry Pi OS Lite
- 🔗 Enabling UART on GPIO 14/15
- 🖥️ Connecting via SSH or USB serial
- 🦀 Installing Rust

### 🔹 2. Wire the PN532

```
🥧 Pi Zero 2WH                    🛸 PN532 Module
┌─────────────────────┐          ┌──────────────┐
│ Pin 1  (3.3V) ──────┼──────────┼── VCC        │
│ Pin 6  (GND)  ──────┼──────────┼── GND        │
│ Pin 10 (RX)   ──────┼──────────┼── TX         │
│ Pin 8  (TX)   ──────┼──────────┼── RX         │
└─────────────────────┘          └──────────────┘
```

| PN532 | Pi GPIO | Header Pin |
|-------|---------|------------|
| ⚡ VCC | 3.3V | Pin 1 |
| ⛓️ GND | GND | Pin 6 |
| 📤 TX | RX (GP15) | Pin 10 |
| 📥 RX | TX (GP14) | Pin 8 |

> ⚠️ **Critical:** VCC → **3.3V** (NOT 5V!). TX→RX is **crossover**.
>
> 📸 See [`docs/HARDWARE_SETUP.md`](docs/HARDWARE_SETUP.md) for detailed diagrams.

### 🔹 3. Build & run

```bash
# Clone on your Pi
git clone https://github.com/arpanpathak/NearField_22x7_rational_communication.git
cd NearField_22x7_rational_communication

# Build (☕ takes a few minutes on Pi Zero)
cargo build --release

# Run
RUST_LOG=info ./target/release/nearfield_22x7_rational_communication
```

### 🔹 4. Tap a tag 🎯

Hold any NFC tag close to the PN532 antenna. You'll see:

```log
[14:32:01.234] [INFO] [UID=7ed59290] Type=Mifare Classic 1K
```

And the ASCII display lights up:

```
┌────────────────────┐
│  📡 NFC TAG        │
│                    │
│ 🔑 UID 7ed59290   │
│ 🏷️ TYPE Mifare 1K │
│ 📍 tap #1          │
└────────────────────┘
```

---

## ⚙️ Configuration

Create a `nearfield.toml` in the working directory:

```toml
serial_port = "/dev/ttyAMA0"     # 📡 PN532 serial port
serial_baud = 115200              # ⚡ Baud rate
db_path = "./nearfield.db"        # 🗄️ SQLite database
poll_interval_ms = 500            # ⏱️ Poll interval
display_type = "stdout"           # 🎨 stdout | eink | pipe | none
max_log_entries = 10000           # 🗑️ Auto-trim limit
```

> All fields are optional. See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) for details.

---

## 📁 Project Structure

```
src/
├── 📄 main.rs           🔁 Entry point + polling loop
├── 📄 config.rs         ⚙️ TOML config loader
├── 📄 error.rs          🚨 Unified error types
├── 📡 nfc/
│   ├── 📄 mod.rs        Re-exports
│   ├── 📄 pn532.rs      PN532 UART protocol driver
│   └── 📄 tag.rs        TagInfo data model
├── 🗄️ storage/
│   ├── 📄 mod.rs        Re-exports
│   ├── 📄 db.rs         SQLite operations
│   └── 📄 models.rs     NfcLogEntry data model
└── 🎨 display/
    ├── 📄 mod.rs        Trait definition
    ├── 📄 ascii.rs      22×7 ASCII art renderer
    └── 📄 backend.rs    Stdout | Pipe | Eink backends
```

---

## 🧪 Testing

```bash
# 🧪 Unit tests (no hardware needed)
cargo test

# 🔬 Integration test (PN532 required)
RUST_LOG=debug cargo run --release
```

> Full test procedures in [`docs/TESTING.md`](docs/TESTING.md)

---

## 📚 Documentation

| Document | What's inside |
|---|---|
| [`docs/OS_SETUP.md`](docs/OS_SETUP.md) | 🖥️ Flashing RPi OS, WiFi, SSH, UART, Rust install, systemd service |
| [`docs/HARDWARE_SETUP.md`](docs/HARDWARE_SETUP.md) | 🔌 Wiring diagrams, pinouts, verification steps |
| [`docs/TESTING.md`](docs/TESTING.md) | 🧪 SSH/USB testing, DB inspection, stress tests |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | 🏗️ Data flow, module map, PN532 protocol ref, extension guide |

---

## 📜 License

**MIT** &nbsp;·&nbsp; See [`LICENSE`](LICENSE)

---

<p align="center">
  <sub>Made with ❤️ by <a href="https://github.com/arpanpathak">@arpanpathak</a></sub>
  <br>
  <sub>🦀 Rust &nbsp;·&nbsp; 🥧 Raspberry Pi &nbsp;·&nbsp; 🛸 NFC &nbsp;·&nbsp; 🎨 ASCII Art</sub>
</p>
