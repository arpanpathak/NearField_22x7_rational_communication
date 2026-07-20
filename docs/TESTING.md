# Testing Guide

> How to test the NFC reader on your Pi — over SSH, USB serial, or a
> desktop machine with a serial device.

---

## 1. Quick Sanity Check

### On the Pi, with tag in hand

```bash
cd ~/nearfield_22x7_rational_communication
RUST_LOG=debug cargo run --release
```

**What to expect after tapping a tag:**

```
[2026-07-20T14:32:01Z INFO  nearfield] Firmware: PN532 v1.6
[2026-07-20T14:32:01Z INFO  nearfield] Entering main loop (poll every 500 ms)
[2026-07-20T14:32:05Z INFO  nearfield] [14:32:05.123] UID=7ed59290 Type=Mifare Classic 1K
[2026-07-20T14:32:05Z INFO  nearfield] Logged tag 7ed59290 (id=1)
```

The ASCII art display will show (with `display_type = "stdout"`):

```
┌────────────────────┐
│  NFC TAG           │
│                    │
│ UID 7ed59290       │
│ TYPE Mifare Classic│
│ tap #1             │
└────────────────────┘
```

### If you get a serial error

```
ERROR nearfield] PN532 not responding: No device found
```

Check:
1. Wiring (see [HARDWARE_SETUP.md](HARDWARE_SETUP.md))
2. UART is enabled (`ls -la /dev/serial0`)
3. Permissions (`sudo usermod -a -G dialout pi`)

---

## 2. Testing Over SSH

This is the most common development workflow — the Pi runs headless,
you connect from your laptop.

### From your laptop

```bash
# SSH into the Pi
ssh pi@raspberrypi.local

# Build and run
cd ~/nearfield_22x7_rational_communication
RUST_LOG=info cargo run --release
```

The stdout backend will print the ASCII frame to the SSH terminal.
Works great for debugging.

### Running in the background with tmux

For long-running tests:

```bash
# Install tmux
sudo apt install tmux

# Start a tmux session
tmux new -s nearfield

# Inside tmux
cd ~/nearfield_22x7_rational_communication
RUST_LOG=info cargo run --release

# Detach: Ctrl-B, D
# Reattach: tmux attach -t nearfield
```

### Using the pipe backend

```bash
# On the Pi: run with pipe backend
NEARFIELD_DISPLAY_TYPE=pipe \
NEARFIELD_DISPLAY_PIPE=/tmp/nearfield.fifo \
RUST_LOG=info cargo run --release &

# On the Pi (or from another SSH session): read from the pipe
cat /tmp/nearfield.fifo
```

Each tap writes a JSON frame to the pipe:

```json
{"type":"frame","width":22,"height":7,"lines":["┌────────────────────┐","│  NFC TAG           │",...]}
```

---

## 3. Testing Over USB Serial (OTG)

If you don't have WiFi or want to debug at the serial console level:

### On the Pi (do this once)

```bash
# Enable USB gadget serial mode
echo "dtoverlay=dwc2" | sudo tee -a /boot/config.txt
echo "g_serial" | sudo tee -a /etc/modules

# Reboot
sudo reboot
```

### On your laptop (macOS)

```bash
# After connecting the Pi via USB (middle USB port):
ls /dev/cu.usbmodem*

# Connect
screen /dev/cu.usbmodem1101 115200

# You now have a serial console — login and run the app
```

### On your laptop (Linux)

```bash
ls /dev/ttyACM*
screen /dev/ttyACM0 115200
```

---

## 4. Testing Without Hardware (Unit Tests)

The ASCII renderer and tag classification work without any hardware:

```bash
cd ~/nearfield_22x7_rational_communication

# Run all unit tests
cargo test

# Run with output
cargo test -- --nocapture
```

### Test output

```
running 4 tests
test display::ascii::tests::test_render_tag_output_size ... ok
test display::ascii::tests::test_render_idle_output_size ... ok
test display::ascii::tests::test_render_history_output_size ... ok
test display::ascii::tests::test_tag_uid_in_output ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## 5. Database Inspection

### On the Pi, after tapping some tags

```bash
# Check the SQLite database
sqlite3 nearfield.db

# In the sqlite3 prompt:
sqlite> .headers on
sqlite> .mode column
sqlite> SELECT id, uid, tag_type, timestamp FROM nfc_logs ORDER BY id DESC LIMIT 5;

id          uid         tag_type              timestamp
----------  ----------  --------------------  ------------------------
1           7ed59290    Mifare Classic 1K     2026-07-20T14:32:05.123Z
```

### Export to JSON

```bash
# Use the built-in export
NEARFIELD_DB_PATH=./nearfield.db cargo run --release -- --export-json > tags.json
```

(Add this flag to `main.rs` — or just query with `sqlite3`:

```bash
sqlite3 nearfield.db -json "SELECT * FROM nfc_logs ORDER BY id DESC LIMIT 10;" > tags.json
```

---

## 6. Testing the E-ink Display (if connected)

### Pipe backend → external display script

The e-ink backend is a stub. To drive a physical e-ink display,
write an external script that reads from the named pipe:

```python
#!/usr/bin/env python3
"""Read frames from the NearField pipe and display on Waveshare e-ink."""
import json
import sys

FIFO_PATH = "/tmp/nearfield.fifo"

print("Waiting for NFC frames...")
with open(FIFO_PATH, "r") as fifo:
    for line in fifo:
        frame = json.loads(line.strip())
        if frame["type"] == "frame":
            for line in frame["lines"]:
                print(line)
            print("─" * 22)
        elif frame["type"] == "clear":
            print("(cleared)")
        sys.stdout.flush()
```

Run it:

```bash
python3 pipe_reader.py
```

Then in another shell:

```bash
NEARFIELD_DISPLAY_TYPE=pipe \
NEARFIELD_DISPLAY_PIPE=/tmp/nearfield.fifo \
RUST_LOG=info cargo run --release
```

---

## 7. Performance Benchmarks

### Polling latency

```bash
# Run with trace logging to see individual frame timings
RUST_LOG=trace cargo run --release 2>&1 | grep "TX\|RX"
```

Typical results on Pi Zero 2WH @ 1 GHz:

| Operation              | Time        |
|------------------------|-------------|
| PN532 SAM config       | ~8 ms       |
| Tag poll (no tag)      | ~15 ms      |
| Tag poll (with tag)    | ~25 ms      |
| SQLite insert          | ~2 ms       |
| ASCII render           | < 0.1 ms    |

### Power consumption

| State                   | Current (approx) |
|-------------------------|-------------------|
| Idle (Pi + PN532)       | ~160 mA @ 5V     |
| Reading tag + DB write  | ~220 mA @ 5V     |
| Pi Zero 2WH alone idle  | ~120 mA @ 5V     |

---

## 8. Common Test Scenarios

### "Tap 100 tags in rapid succession"

```bash
# Run a quick stress test by tapping tags rapidly
# (or use another NFC reader to replay UIDs)
timeout 60 cargo run --release
```

The 1-second debounce prevents duplicate entries. Each unique tap
within the debounce window is logged exactly once.

### "Reconnect test — unplug and replug the PN532"

```bash
# While the app is running, disconnect the PN532's USB/power
# Then reconnect it. The app should auto-reconnect:
```

Log output should show:

```
[ERROR] Poll error: Read timeout after 0/6 bytes
[INFO] Attempting PN532 re-initialisation...
[INFO] Serial port /dev/ttyAMA0 opened at 115200 baud
[INFO] PN532 re-initialised successfully
```

### "What tags are supported?"

Hold different tags to the reader. The `tag_type` field will show:

| Tag Type                     | ATQA     | SAK |
|------------------------------|----------|-----|
| Mifare Classic 1K            | 00 04    | 08  |
| Mifare Classic 4K            | 00 02    | 18  |
| Mifare Ultralight            | 00 44    | 00  |
| NTAG213/215/216              | 00 44    | 20  |
| Mifare DESFire               | 00 03    | 20  |
| ISO/IEC 14443-4 Compliant    | varies   | bit5 set |

Unknown tags will be reported as `"Unknown Type-A (ATQA=XXYY, SAK=ZZ)"`.
If you discover a new tag type, open a PR to add it to `tag.rs`.
