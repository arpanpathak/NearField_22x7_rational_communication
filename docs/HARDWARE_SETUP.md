# Hardware Setup Guide

> **Wiring the PN532 NFC module to your Raspberry Pi Zero 2WH.**
> Includes pinout diagrams, wiring steps, and verification.

---

## 1. PN532 Module Pinout

A typical PN532 module (from Elechouse, ITEAD, or generic) has these pins:

```
┌─────────────────────────────┐
│  PN532 NFC Module            │
│                              │
│  ┌───┬───┬───┬───┬───┬───┐ │
│  │VCC│GND│TX │RX │SCL│SDA│ │
│  └───┴───┴───┴───┴───┴───┘ │
│                              │
│  ┌───┬───┬───┬───┬───┬───┐ │
│  │NSS│SCK│MOS│MIS│RST│IRQ│ │
│  └───┴───┴───┴───┴───┴───┘ │
│                              │
│   [Antenna Area]            │
└─────────────────────────────┘
```

For **UART mode** we only need 4 pins: **VCC, GND, TX, RX**.

### Setting UART mode

Most PN532 modules auto-detect the interface. If yours doesn't:

1. Check if there's a **DIP switch** or **jumper** labelled "UART/I2C/SPI".
2. Set it to **UART** (or leave all switches OFF for auto-detect).

---

## 2. Wiring Diagram

### Pi Zero 2WH GPIO Header (J8, 40-pin)

```
            ┌──────────────────────┐
            │  Pi Zero 2WH Header  │
            │  (top view, SD card  │
            │   on the left)       │
            │                      │
  3.3V ──── │  1  2 │ ──── 5V     │
  GPIO2 ─── │  3  4 │ ──── 5V     │
  GPIO3 ─── │  5  6 │ ──── GND ── │ ───── PN532 GND
  GPIO4 ─── │  7  8 │ ──── TX ─── │ ───── PN532 RX (GP14)
     GND ── │  9 10 │ ──── RX ─── │ ───── PN532 TX (GP15)
            │ ...  │              │
            └──────────────────────┘
```

### Connection Table

| PN532 Pin | Pi GPIO Pin | Wire Colour (example) |
|-----------|-------------|-----------------------|
| **VCC**   | **Pin 1** (3.3V) | Red     |
| **GND**   | **Pin 6** (GND)  | Black   |
| **TX**    | **Pin 10** (RX / GPIO15) | White |
| **RX**    | **Pin 8** (TX / GPIO14)  | Green  |

### Critical Notes

- **PN532 VCC → 3.3V (NOT 5V).** Connecting VCC to 5V will damage the PN532.
- **TX→RX crossover.** PN532 TX goes to Pi RX (GPIO15 / Pin 10),
  and PN532 RX goes to Pi TX (GPIO14 / Pin 8).
- **Common ground.** Always connect GND between both devices.

---

## 3. Step-by-Step Wiring

### Tools needed

- PN532 module with 4 pins soldered (you already did this)
- 4× female-to-female Dupont jumper wires
- Raspberry Pi Zero 2WH

### Steps

1. **Power off** the Pi (disconnect Micro-USB power).

2. **Connect GND first:**
   - Black wire: PN532 GND → Pi Pin 6 (GND)

3. **Connect VCC:**
   - Red wire: PN532 VCC → Pi Pin 1 (3.3V)

4. **Connect TX→RX (crossover):**
   - White wire: PN532 TX → Pi Pin 10 (RX / GPIO15)

5. **Connect RX→TX (crossover):**
   - Green wire: PN532 RX → Pi Pin 8 (TX / GPIO14)

6. **Double-check** every connection before applying power.

7. **Power on** the Pi.

---

## 4. Verify the Connection

### Check power

On a properly powered PN532, the red **PWR** LED should be lit.

### Check serial communication

Install `minicom` or `screen` and listen on the UART:

```bash
# Install minicom
sudo apt install minicom

# Listen on the UART (you'll see garbage bytes when PN532 wakes up)
minicom -b 115200 -D /dev/serial0
```

Press Ctrl-A, then X to quit minicom.

### Run a quick test with libnfc (optional)

```bash
sudo apt install libnfc-bin

# Check if the PN532 is detected
sudo nfc-list
```

Expected output:

```
nfc-list uses libnfc 1.8.0
NFC device: pn532_uart:/dev/ttyAMA0 opened
1 ISO14443A passive target(s) found:
ISO/IEC 14443A (106 kbps) target:
    ATQA (SENS_RES): 00  04
    UID (NFCID1): 7e  d5  92  90
    SAK (SEL_RES): 08
```

This confirms:
- UART is working
- PN532 is responding
- Tags can be detected

Hold a tag near the antenna to see it listed.

---

## 5. E-ink Display (Optional)

If you want to add a Waveshare e-ink display:

### Wiring (Waveshare 2.13" HAT)

| E-ink Pin | Pi GPIO     | Pi Pin |
|-----------|-------------|--------|
| VCC       | 3.3V        | Pin 1  |
| GND       | GND         | Pin 6  |
| DIN (MOSI)| GPIO10(MOSI)| Pin 19 |
| CLK (SCLK)| GPIO11(SCLK)| Pin 23 |
| CS        | GPIO8 (CE0) | Pin 24 |
| DC        | GPIO25      | Pin 22 |
| RST       | GPIO17      | Pin 11 |
| BUSY      | GPIO24      | Pin 18 |

> **Note:** The e-ink display is a secondary component. The NFC reader
> works independently of the display. Start with just the PN532.

---

## 6. Antenna Range

The PN532's on-board antenna can read:

- **Mifare Classic cards:** ~3-5 cm
- **NTAG stickers:** ~2-3 cm
- **Phone NFC:** ~1-2 cm

If you need longer range, you can:

1. Remove the on-board antenna jumper (if present).
2. Solder an external 13.56 MHz antenna tuned to 1-2 µH.

---

## 7. Pinout Quick Reference Cards

### Pi Zero 2WH GPIO Reference

```
                    Pi Zero 2 WH GPIO (40-pin)
                    ┌──────────────────────┐
                    │ 3V3  │ 1   2 │ 5V     │
                    │ GPIO2│ 3   4 │ 5V     │
                    │ GPIO3│ 5   6 │ GND    │
                    │ GPIO4│ 7   8 │ GPIO14 │←─ TX to PN532 RX
                    │ GND  │ 9  10 │ GPIO15 │←─ RX from PN532 TX
                    │GPIO17│11  12 │ GPIO18 │
                    │GPIO27│13  14 │ GND    │
                    │GPIO22│15  16 │ GPIO23 │
                    │ 3V3  │17  18 │ GPIO24 │
                    │GPIO10│19  20 │ GND    │←─ SPI MOSI
                    │ GPIO9│21  22 │ GPIO25 │
                    │GPIO11│23  24 │ GPIO8  │←─ SPI CE0
                    │ GND  │25  26 │ GPIO7  │
                    │GPIO0 │27  28 │ GPIO1  │
                    │GPIO5 │29  30 │ GND    │
                    │GPIO6 │31  32 │ GPIO12 │
                    │GPIO13│33  34 │ GND    │
                    │GPIO19│35  36 │ GPIO16 │
                    │GPIO26│37  38 │ GPIO20 │
                    │ GND  │39  40 │ GPIO21 │
                    └──────────────────────┘
```

### PN532 Module Reference

```
PN532 UART pins (top side):

  ┌──────┬──────┬──────┬──────┬──────┬──────┐
  │ VCC  │ GND  │ TX   │ RX   │ SCL  │ SDA  │
  │ (3.3)│      │→PiRX │←PiTX │ (I2C)│ (I2C)│
  └──────┴──────┴──────┴──────┴──────┴──────┘

  ┌──────┬──────┬──────┬──────┬──────┬──────┐
  │ NSS  │ SCK  │ MOSI │ MISO │ RST  │ IRQ  │
  │ (SPI)│ (SPI)│ (SPI)│ (SPI)│      │      │
  └──────┴──────┴──────┴──────┴──────┴──────┘
```
