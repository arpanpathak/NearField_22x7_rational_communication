
<p align="center">
  <img src="https://img.shields.io/badge/rust-1.85%2B-orange?logo=rust" alt="Rust">
  <img src="https://img.shields.io/badge/pi-zero_2wh-success?logo=raspberrypi" alt="Pi Zero">
  <img src="https://img.shields.io/badge/PN532-UART-blue" alt="PN532">
  <img src="https://img.shields.io/badge/license-MIT-green" alt="MIT">
  <img src="https://img.shields.io/badge/PRs-welcome-brightgreen" alt="PRs Welcome">
</p>

<p align="center">
  <code>22/7 ≈ π</code>
</p>

<h1 align="center">📡 NearField_22x7_rational_communication</h1>

<p align="center">
  <b>Low-power NFC tag reader</b> for <b>Raspberry Pi Zero 2WH</b> + <b>PN532</b>
  <br>
  🗄️ Logs taps to SQLite &nbsp;·&nbsp; 🎨 Renders ASCII art on e-ink &nbsp;·&nbsp; 🔋 Runs at ~0.8 W
</p>

<br>

<div align="center">

```
┌────────────────────┐
│  NFC TAG DETECTED  │
│                    │
│ UID 7ed59290       │
│ TYPE Mifare Classic│
│ tap #1             │
└────────────────────┘
```

</div>

<br>

> **🤔 Why the name?** `22/7` is the classic rational approximation of **π** (pi). This project runs on a **Pi** Zero, reads **NFC** tags, and is all about **rational communication** between devices. The display grid just happens to be 22 columns × 7 rows. Serendipity. ✨

---

## ✨ Features

| 🌟 Feature | 💡 Description |
|---|---|
| **📡 Read NFC tags** | PN532 chipset over UART. Supports Mifare Classic, Ultralight, NTAG, DESFire, ISO 14443-4 |
| **🗄️ SQLite logging** | Every tap stored with UID, type, timestamp. Queryable. Exportable to JSON |
| **🎨 ASCII art display** | 22×7 character grid rendered to stdout, named pipe, or e-ink |
| **🔋 Low power** | 500 ms poll interval. PN532 sleeps between polls. Pi Zero sips ~0.8 W |
| **🛡️ Debounced** | 1-second window prevents duplicate logs while tag is held to reader |
| **🚦 Graceful shutdown** | Ctrl-C handler flushes DB and clears display |
| **🔄 Auto-reconnect** | Re-initialises PN532 if serial communication drops |
| **🗑️ Log rotation** | Configurable max entries keeps the database lean |

---

## 🧰 Hardware Requirements

| Component | Spec |
|---|---|
| 🥧 **Raspberry Pi Zero 2WH** | With pre-soldered GPIO header |
| 🛸 **PN532 NFC Module** | Any variant with UART pins |
| 🔌 **4× Dupont wires** (F/F) | VCC, GND, TX, RX |
| 💾 **MicroSD card** | ≥ 8 GB, Class 10 |
| ⚡ **Power supply** | Micro-USB, 2.5 A |

---

## 🚀 Quick Start

### 🔹 1. Set up your Pi

Flash **Raspberry Pi OS Lite (64-bit)** and configure:

| Step | Tool | What to do |
|---|---|---|
| 💿 **Flash OS** | Raspberry Pi Imager | Pick RPi Zero 2W + OS Lite |
| 📶 **WiFi + SSH** | Imager ⚙ settings | Pre-configure before flashing |
| 🔗 **Enable UART** | `sudo raspi-config` | Interface → Serial → No login → Yes hardware |
| 🦀 **Install Rust** | `curl ... sh.rustup.rs` | Default installation |

> Full guide: [`docs/OS_SETUP.md`](docs/OS_SETUP.md)

### 🔹 2. Wire the PN532

```
  🥧 Pi Zero 2WH                          🛸 PN532 Module
  ┌─────────────────────┐                ┌──────────────┐
  │ Pin  1 (3.3V) ──────┼────────────────┼── VCC        │
  │ Pin  6 (GND)  ──────┼────────────────┼── GND        │
  │ Pin 10 (RX)   ──────┼────────────────┼── TX         │
  │ Pin  8 (TX)   ──────┼────────────────┼── RX         │
  └─────────────────────┘                └──────────────┘
```

| PN532 Pin | Pi Connection | Header Pin |
|---|---|---|
| ⚡ VCC | 3.3V | Pin 1 |
| ⛓️ GND | GND | Pin 6 |
| 📤 TX | RX (GPIO 15) | Pin 10 |
| 📥 RX | TX (GPIO 14) | Pin 8 |

> ⚠️ **Critical:** VCC goes to **3.3V** only (NOT 5V!). TX and RX must be **crossed** between devices.

> 📸 See [`docs/HARDWARE_SETUP.md`](docs/HARDWARE_SETUP.md) for detailed diagrams and photos.

### 🔹 3. Build & Run

```bash
# Clone the repo on your Pi
git clone https://github.com/arpanpathak/NearField_22x7_rational_communication.git
cd NearField_22x7_rational_communication

# Build release binary (takes ~5 min on Pi Zero)
cargo build --release

# Run with default settings
RUST_LOG=info ./target/release/nearfield_22x7_rational_communication
```

### 🔹 4. Tap a Tag 🎯

Hold any NFC tag near the PN532 antenna. You'll see:

```
[14:32:01.234] [INFO] [UID=7ed59290] Type=Mifare Classic 1K
```

And the ASCII display shows:

```
┌────────────────────┐
│  NFC TAG DETECTED  │
│                    │
│ UID 7ed59290       │
│ TYPE Mifare Classic│
│ tap #1             │
└────────────────────┘
```

---

## ⚙️ Configuration

Create `nearfield.toml` in the working directory:

```toml
serial_port     = "/dev/ttyAMA0"       # 📡 PN532 serial port path
serial_baud     = 115200               # ⚡ UART baud rate
db_path         = "./nearfield.db"     # 🗄️ SQLite database file path
poll_interval_ms = 500                 # ⏱️  Tag poll interval (milliseconds)
display_type    = "stdout"             # 🎨  stdout | eink | pipe | none
max_log_entries = 10000                # 🗑️  Auto-trim limit (0 = unlimited)
```

> All fields have sensible defaults. See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).

---

## 📁 Project Structure

```
src/
├── main.rs           Entry point + polling loop
├── config.rs         TOML / env-var configuration
├── error.rs          Unified AppError types
│
├── nfc/              PN532 NFC protocol driver
│   ├── pn532.rs      UART frame encoding / decoding
│   └── tag.rs        TagInfo model + type classification
│
├── storage/          SQLite persistence layer
│   ├── db.rs         Connection, migrations, CRUD
│   └── models.rs     NfcLogEntry data model
│
└── display/          Output rendering
    ├── ascii.rs      22x7 ASCII art grid renderer
    └── backend.rs    Stdout | Pipe | Eink backends
```

---

## 🧪 Testing

```bash
# Unit tests (no hardware required)
cargo test

# Integration test (requires connected PN532)
RUST_LOG=debug cargo run --release
```

> Full testing procedures: [`docs/TESTING.md`](docs/TESTING.md)

---

## 📚 Documentation

| Document | Description |
|---|---|
| [`docs/OS_SETUP.md`](docs/OS_SETUP.md) | 🖥️ Flashing, WiFi, SSH, UART, Rust, systemd service |
| [`docs/HARDWARE_SETUP.md`](docs/HARDWARE_SETUP.md) | 🔌 Wiring diagrams, pinouts, verification |
| [`docs/TESTING.md`](docs/TESTING.md) | 🧪 Testing over SSH, USB, DB inspection, benchmarks |
| [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) | 🏗️ Data flow, module map, PN532 protocol ref |
| [`CONTRIBUTING.md`](CONTRIBUTING.md) | 🤝 How to contribute, code style, PR workflow |
| [`SECURITY.md`](SECURITY.md) | 🔒 Security policies and best practices |

---

## 🤝 Contributing

Contributions are welcome! Please see [`CONTRIBUTING.md`](CONTRIBUTING.md) for guidelines.

Quick checklist:
1. 🍴 Fork the repo
2. 🌿 Create a feature branch
3. ✏️ Make your changes
4. 🧪 Ensure `cargo test` passes
5. 📬 Open a Pull Request

---

## 🔒 Security

See [`SECURITY.md`](SECURITY.md) for our security policy and best practices for embedded Rust devices.

---

## 📜 License

**MIT** &nbsp;·&nbsp; See [`LICENSE`](LICENSE)

---

<p align="center">
  <sub>Made with ❤️ by <a href="https://github.com/arpanpathak">@arpanpathak</a></sub>
  <br>
  <sub>🦀 Rust &nbsp;·&nbsp; 🥧 Raspberry Pi &nbsp;·&nbsp; 🛸 NFC &nbsp;·&nbsp; 🎨 ASCII Art</sub>
</p>
