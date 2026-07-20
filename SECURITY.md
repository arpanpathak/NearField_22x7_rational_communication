
# 🔒 Security Policy & Best Practices

## Supported Versions

| Version | Supported |
|---|---|
| v0.1.x  | ✅ Active development — security patches guaranteed |

To report a vulnerability, **do not** open a public GitHub issue.
Instead, email the maintainer or open a [draft security advisory](https://github.com/arpanpathak/NearField_22x7_rational_communication/security/advisories/new).

---

## 📋 Table of Contents

- [Reporting a Vulnerability](#reporting-a-vulnerability)
- [Security Posture](#security-posture)
- [Best Practices for Embedded NFC Devices](#best-practices-for-embedded-nfc-devices)
  - [1. Physical Security](#1-physical-security)
  - [2. Network Security](#2-network-security)
  - [3. NFC / RF Security](#3-nfc--rf-security)
  - [4. Data Storage Security](#4-data-storage-security)
  - [5. Software Supply Chain](#5-software-supply-chain)
  - [6. Operational Security](#6-operational-security)
  - [7. Firmware & Update Security](#7-firmware--update-security)
- [Security Checklist](#security-checklist)

---

## Reporting a Vulnerability

If you discover a security vulnerability, please:

1. **Do not** disclose it publicly until we've had a chance to respond.
2. Email the maintainers or create a [draft advisory](https://github.com/arpanpathak/NearField_22x7_rational_communication/security/advisories/new).
3. Include:
   - A description of the vulnerability
   - Steps to reproduce
   - Affected versions
   - Any potential impact

We aim to acknowledge receipt within **48 hours** and provide a fix timeline
within **5 business days**.

---

## Security Posture

This project operates in a **trusted, local environment** — the NFC reader
sits on your desk or wall, connected to your home WiFi. It is **not**
designed to be internet-facing or handle sensitive credentials
(credit cards, government IDs, etc.).

However, security still matters. Here's our approach:

| Area | Stance |
|---|---|
| 🏠 **Deployment environment** | Local / home network only |
| 📡 **NFC data** | Read-only (UID + type). No credential storage |
| 🌐 **Network exposure** | No open ports. SSH-only access |
| 🗄️ **Database** | Local SQLite file. No remote access |
| 🦀 **Memory safety** | Rust eliminates whole classes of memory bugs |

---

## Best Practices for Embedded NFC Devices

### 1. 🔐 Physical Security

The Pi Zero 2WH is a small, exposed device. Protect it:

| Practice | Why |
|---|---|
| **Enclose the device** | 3D-printed or off-the-shelf case prevents physical tampering |
| **Disable USB OTG** | Prevents unauthorized USB gadget access. Add `dtoverlay=disable-otg` to `/boot/config.txt` if unused |
| **Disable JTAG/SWD** | Not exposed on Pi Zero, but worth noting for ARM devices |
| **Lock the SD card slot** | Use a case that covers the slot. Or solder the SD card internally |
| **Use a tamper-evident seal** | Cheap stickers that show if the case was opened |
| **Disable Bluetooth** | `sudo systemctl disable bluetooth` — reduces attack surface |

### 2. 🌐 Network Security

| Practice | How |
|---|---|
| **Use a separate IoT VLAN** | Isolate the Pi from your main devices. Most home routers support this |
| **Disable WiFi power save** | `sudo iw dev wlan0 set power_save off` — avoids connectivity drops |
| **Use a static IP or DHCP reservation** | Prevents MITM via rogue DHCP |
| **Disable unnecessary services** | `sudo systemctl disable triggerhappy avahi-daemon cron` |
| **Change default password** | `passwd` — don't leave `raspberry` as the password |
| **Use SSH keys only** | No password auth: `/etc/ssh/sshd_config` → `PasswordAuthentication no` |
| **Restrict SSH to local network** | Don't port-forward port 22 on your router |
| **Use a firewall** | `sudo apt install ufw && sudo ufw allow ssh && sudo ufw enable` |
| **Audit open ports** | `sudo netstat -tulpn` — nothing should listen except SSH |

### 3. 📡 NFC / RF Security

NFC is inherently short-range (~4 cm). The main risks are:

| Risk | Mitigation |
|---|---|
| **Eavesdropping** (skimming UIDs) | NFC range is ~4 cm. Attackers need physical proximity |
| **Relay attack** (extending range) | Use cryptographic tags (DESFire) if relay is a concern |
| **Tag spoofing** (fake UIDs) | This reader logs **raw UID** + **ATQA/SAK**. Cloned tags show identical data but are detectable via cryptographic authentication |
| **Denial of service** (RF jamming) | 13.56 MHz jammers exist but are bulky and illegal in most countries |

**Recommendations for sensitive deployments:**

- Use **Mifare DESFire** or **NTAG 424 DNA** tags with mutual authentication
- The PN532 supports cipher operations — extend `TagInfo` to include
  cryptographic verification
- Log failed authentication attempts to detect brute-force scanning

### 4. 🗄️ Data Storage Security

| Practice | Implementation |
|---|---|
| **Database is local-only** | SQLite file on the Pi's SD card. No network exposure |
| **No credentials stored** | We log only UID, tag type, and timestamp |
| **Disk encryption (optional)** | Full-disk encryption with LUKS on Raspberry Pi OS is possible but complex |
| **Auto-trim old logs** | `max_log_entries` config option prevents unbounded growth |
| **Regular backups** | `cp nearfield.db nearfield.db.bak` — cheap insurance |
| **WAL mode** | Enabled by default. Prevents DB corruption on power loss |
| **JSON export** | Use `Database::export_json()` for safe data portability |

### 5. 📦 Software Supply Chain

| Practice | How |
|---|---|
| **Pin dependency versions** | `Cargo.lock` is committed to the repo. Use it |
| **Audit dependencies** | `cargo audit` — checks for known vulnerabilities in crates |
| **Review `unsafe` code** | This project currently has **zero `unsafe` blocks** |
| **Use `opt-level = "s"`** | Release builds are size-optimised — harder to reverse-engineer |
| **Strip symbols** | Release builds have `strip = true` in `Cargo.toml` |
| **Verify checksums** | `sha256sum nearfield_db` before copying over network |

Install `cargo-audit`:

```bash
cargo install cargo-audit
cargo audit  # Run this before every release
```

### 6. ⚙️ Operational Security

| Practice | How |
|---|---|
| **Run as non-root** | Default `pi` user. DO NOT run as root |
| **Use systemd hardening** | Add to `/etc/systemd/system/nearfield.service`: |

```ini
[Service]
ProtectSystem=strict
ProtectHome=true
PrivateTmp=true
NoNewPrivileges=true
CapabilityBoundingSet=~CAP_NET_RAW ~CAP_NET_ADMIN
```

| Practice | How |
|---|---|
| **Monitor logs** | `journalctl -u nearfield.service -f` |
| **Set up alerts** | Simple cron job that checks `nearfield.db` growth |
| **Regular updates** | `sudo apt update && sudo apt upgrade -y` monthly |
| **Read-only filesystem (advanced)** | `sudo raspi-config` → Performance → Overlay Filesystem → Enable. Protects against SD card corruption and tampering |

### 7. 🔄 Firmware & Update Security

| Practice | How |
|---|---|
| **Pull from tagged releases** | `git checkout v0.1.0` — not random commits |
| **Verify git tags** | `git tag -v v0.1.0` — signed tags (once GPG signing is set up) |
| **Build from source** | `cargo build --release` — reproducible builds |
| **Test updates offline first** | Run new binary with `--dry-run` or on a test SD card |
| **Rollback plan** | Keep the previous binary: `cp target/release/nearfield ~/nearfield.bak` |

---

## Security Checklist

Use this checklist when deploying the device:

- [ ] 🏠 Device is in a locked / enclosed case
- [ ] 🌐 Pi is on an isolated VLAN or firewall-restricted subnet
- [ ] 🔑 Default `pi` password changed
- [ ] 🔐 SSH key authentication only (no passwords)
- [ ] 🛡️ `ufw` enabled — only SSH port open
- [ ] 🚫 Bluetooth disabled
- [ ] 🧹 Unnecessary services disabled (triggerhappy, avahi-daemon, cron)
- [ ] 🗄️ `max_log_entries` configured
- [ ] 🦀 `cargo audit` run — zero vulnerabilities
- [ ] 📦 `Cargo.lock` up to date
- [ ] 📡 NFC antenna physically secured (can't be easily removed)
- [ ] 🔄 SD card backup taken
- [ ] 📈 Log monitoring set up
- [ ] 🚦 systemd service has `ProtectSystem=strict` and `NoNewPrivileges=true`

---

> **Remember:** Security is a process, not a product. Review these practices
> regularly and update them as the threat landscape evolves.
>
> *"The only truly secure system is one that is powered off, cast in a block of
> concrete and sealed in a lead-lined room with armed guards."* — Gene Spafford
>
> Everything else is about **risk management**. 🛡️
