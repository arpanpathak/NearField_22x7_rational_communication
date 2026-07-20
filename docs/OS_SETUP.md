# Raspberry Pi OS Setup Guide

> Complete walkthrough: from a blank MicroSD to a running Rust NFC reader.
> Covers **headless** setup (no keyboard, no monitor — just your laptop).

---

## 1. Flash Raspberry Pi OS

### What you need

- MicroSD card (≥ 8 GB, Class 10 recommended)
- SD card reader/writer
- [Raspberry Pi Imager](https://www.raspberrypi.com/software/) (or `dd`)

### Using Raspberry Pi Imager (easiest)

1. Launch Raspberry Pi Imager.
2. Click **CHOOSE DEVICE** → **Raspberry Pi Zero 2 W**.
3. Click **CHOOSE OS** → **Raspberry Pi OS (other)** → **Raspberry Pi OS Lite (64-bit)**.
   - Use **Lite** — no desktop, saves power and storage.
4. Click **CHOOSE STORAGE** → select your SD card.
5. Click the gear icon (⚙) to pre-configure:
   - [x] **Enable SSH** — set a password for user `pi`.
   - [x] **Set username & password** — e.g. `pi` / `yourpassword`.
   - [x] **Configure wireless LAN** — enter your WiFi SSID and password.
     - Set **Wireless LAN country** to your country code (e.g. `US`, `GB`, `IN`).
   - [x] **Set locale settings** — timezone (e.g. `Asia/Kolkata`), keyboard layout.
6. Click **WRITE** → wait for it to finish → eject the SD card.

### Alternative: Manual setup with `dd`

```bash
# Download the image
wget https://downloads.raspberrypi.com/raspios_lite_arm64/images/.../image_2026-XX-XX-raspios-bookworm-arm64-lite.img.xz

# Flash to SD card (find your device with `lsblk` or `diskutil list`)
xzcat image_2026-XX-XX-raspios-bookworm-arm64-lite.img.xz | sudo dd of=/dev/mmcblk0 bs=4M status=progress
sync

# Mount the boot partition and enable SSH + WiFi
mount /dev/mmcblk0p1 /mnt/boot
touch /mnt/boot/ssh

cat > /mnt/boot/wpa_supplicant.conf << 'EOF'
ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev
update_config=1
country=IN

network={
    ssid="YourWiFiSSID"
    psk="YourWiFiPassword"
    key_mgmt=WPA-PSK
}
EOF

sync
umount /mnt/boot
```

---

## 2. Boot and Connect

1. Insert the MicroSD into your Pi Zero 2WH.
2. Connect Micro-USB power (use the **USB** port, not the OTG port).
3. Wait ~30 seconds for boot.
4. Find the Pi on your network:

```bash
# From your laptop
ping raspberrypi.local
# or use nmap
nmap -sn 192.168.1.0/24  # Scan your local subnet
```

5. SSH in:

```bash
ssh pi@raspberrypi.local
# Password: the one you set in Raspberry Pi Imager
```

### If you can't find it via mDNS

Connect the Pi to your laptop via **USB OTG** (the middle USB port on the
Pi Zero) — it shows up as a serial device:

```bash
# On macOS: check for a new serial device
ls /dev/cu.usbmodem*

# Connect at 115200 baud (you may need `screen` or `minicom`)
screen /dev/cu.usbmodem1101 115200
```

> **Note:** USB serial (OTG) requires `dtoverlay=dwc2` and `g_serial` in
> `/boot/config.txt`. The Raspberry Pi Imager can configure this for you,
> or you can do it manually after the first SSH session.

---

## 3. Initial Configuration

### Update the system

```bash
sudo apt update && sudo apt full-upgrade -y
sudo apt install -y git curl build-essential pkg-config libssl-dev
sudo reboot
```

### Enable UART (for PN532)

UART is disabled by default on the Pi Zero 2WH. Enable it:

```bash
sudo raspi-config
```

Navigate:
1. **Interface Options** → **Serial Port**
2. → "Would you like a login shell to be accessible over serial?" → **No**
3. → "Would you like the serial port hardware to be enabled?" → **Yes**
4. → **Finish** → **Yes** to reboot.

This configures:
- `/dev/ttyAMA0` available for the PN532
- No login shell on the serial pins (GPIO 14/15)
- `/dev/serial0` symlinked to `/dev/ttyAMA0`

**Verify UART is working:**

```bash
# Check the device exists
ls -la /dev/serial*

# Should show: /dev/serial0 -> ttyAMA0
# If you get "No such file or directory", UART is not enabled.
```

### (Optional) Enable SPI for e-ink display

If you plan to use a Waveshare e-ink display:

```bash
sudo raspi-config
# Interface Options → SPI → Yes → Finish → Reboot
```

Verify:

```bash
ls -la /dev/spidev*
# Should show: /dev/spidev0.0  /dev/spidev0.1
```

### (Optional) Enable I2C

Some PN532 modules can also run over I2C. If you prefer I2C over UART:

```bash
sudo raspi-config
# Interface Options → I2C → Yes → Finish → Reboot
```

---

## 4. Install Rust

The easiest way is via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

Choose **default installation** when prompted.

**This takes ~5 minutes on a Pi Zero 2WH.** Rustup compiles from source
on ARM devices. Grab a coffee.

Verify:

```bash
rustc --version   # Should show rustc 1.xx.0
cargo --version   # Should show cargo 1.xx.0
```

### Install build dependencies

```bash
sudo apt install -y libsqlite3-dev
```

(Only needed if you disable the `bundled` feature of `rusqlite`. With the
default `bundled` feature, SQLite is compiled from source automatically.)

---

## 5. Clone and Build the Project

```bash
git clone https://github.com/arpanpathak/nearfield_22x7_rational_communication.git
cd nearfield_22x7_rational_communication

# Build (this takes a while on Pi Zero — ~10-15 minutes for release build)
cargo build --release
```

**Build a binary for faster startup next time:**

```bash
# Copy the binary somewhere convenient
cp target/release/nearfield_22x7_rational_communication ~/nearfield
```

---

## 6. Run as a Service (Autostart on Boot)

Create a systemd service to launch the reader automatically:

```bash
sudo nano /etc/systemd/system/nearfield.service
```

```ini
[Unit]
Description=NearField NFC Tag Reader
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=pi
WorkingDirectory=/home/pi
ExecStart=/home/pi/nearfield_22x7_rational_communication/target/release/nearfield_22x7_rational_communication
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=NEARFIELD_DISPLAY_TYPE=stdout
Environment=NEARFIELD_SERIAL_PORT=/dev/ttyAMA0

[Install]
WantedBy=multi-user.target
```

Enable and start:

```bash
sudo systemctl enable nearfield.service
sudo systemctl start nearfield.service
sudo systemctl status nearfield.service
```

View logs:

```bash
journalctl -u nearfield.service -f
```

---

## 7. Power Optimisation (Optional)

The Pi Zero 2WH is already frugal (~0.8 W idle), but you can optimise further:

```bash
# Disable HDMI (saves ~25 mA)
sudo tvservice -o

# Disable Bluetooth (if not needed)
sudo systemctl disable bluetooth

# Disable Wi-Fi power saving
sudo iw dev wlan0 set power_save off

# Reduce GPU memory
echo "gpu_mem=16" | sudo tee -a /boot/config.txt

# Disable unnecessary services
sudo systemctl disable triggerhappy
sudo systemctl disable avahi-daemon
sudo systemctl disable cron
```

---

## 8. Troubleshooting

| Symptom | Likely Cause | Fix |
|---------|-------------|-----|
| `/dev/serial0` missing | UART not enabled | Run `sudo raspi-config`, enable Serial Port |
| `Permission denied` on serial port | User not in `dialout` group | `sudo usermod -a -G dialout pi` then logout/login |
| PN532 not responding | Wrong wiring | Check VCC→3.3V (NOT 5V!), TX↔RX crossover |
| `Connection refused` on SSH | SSH not enabled | Mount SD card, `touch boot/ssh` |
| Rust build takes forever | Pi Zero is ARMv8 @ 1 GHz | Normal; use `cargo build --release` only once |
| `rustup: not found` | Need to source profile | `source $HOME/.cargo/env` |
