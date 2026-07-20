#!/usr/bin/env bash
#
# setup.sh — One-command automated setup for the NearField NFC reader on Pi.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/arpanpathak/nearfield_22x7_rational_communication/main/scripts/setup.sh | bash
#
# Or locally:
#   ./scripts/setup.sh [--service] [--build-only]
#
# What it does:
#   1. Detects Pi model & OS
#   2. Enables UART (edits /boot/config.txt directly — no raspi-config needed)
#   3. Installs system packages (build-essential, pkg-config, libssl-dev, etc.)
#   4. Installs Rust via rustup (if not already installed)
#   5. Adds user to dialout group for serial port access
#   6. Clones or updates the project repo
#   7. Builds the release binary
#   8. Optionally installs systemd service for auto-start on boot
#
# Idempotent: safe to re-run. Skips steps already completed.

set -euo pipefail

# ── Colour helpers ─────────────────────────────────────────────────────────
RED='\033[0;31m';  GREEN='\033[0;32m';  YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m';       NC='\033[0m'
info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
warn()  { echo -e "${YELLOW}[WARN]${NC}  $*"; }
err()   { echo -e "${RED}[ERR]${NC}   $*" >&2; }

# ── Defaults ───────────────────────────────────────────────────────────────
INSTALL_SERVICE=false
BUILD_ONLY=false
REPO_URL="https://github.com/arpanpathak/nearfield_22x7_rational_communication.git"
REPO_DIR="$HOME/nearfield_22x7_rational_communication"
BIN_NAME="nearfield_22x7_rational_communication"
SERVICE_NAME="nearfield.service"

# ── Parse flags ────────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --service)    INSTALL_SERVICE=true;  shift ;;
        --build-only) BUILD_ONLY=true;       shift ;;
        --help|-h)
            echo "Usage: $0 [--service] [--build-only]"
            echo ""
            echo "  --service     Install systemd service for auto-start on boot"
            echo "  --build-only  Only build the binary (skip system setup)"
            echo "  --help        Show this help"
            exit 0
            ;;
        *) err "Unknown option: $1"; exit 1 ;;
    esac
done

# ── Pre-flight checks ──────────────────────────────────────────────────────

# Must be run on a Pi
if [[ ! -f /proc/device-tree/model ]]; then
    err "This script must be run on a Raspberry Pi."
    exit 1
fi
PI_MODEL=$(tr -d '\0' < /proc/device-tree/model)
info "Detected: ${PI_MODEL}"

# Must be run as pi (or a non-root user with sudo)
if [[ "$(id -u)" -eq 0 ]]; then
    err "Do NOT run this script as root. Run it as your normal user (e.g. pi)."
    exit 1
fi
if ! command -v sudo &>/dev/null; then
    err "sudo is required but not installed."
    exit 1
fi

# ── Step 1: System packages & updates ────────────────────────────────────
step_system_packages() {
    echo ""
    info "${BOLD}Step 1/6:${NC} Updating system packages..."

    sudo apt update -qq
    sudo apt full-upgrade -y -qq
    sudo apt install -y -qq \
        git \
        curl \
        build-essential \
        pkg-config \
        libssl-dev \
        ca-certificates

    ok "System packages updated and installed."
}

# ── Step 2: Enable UART ───────────────────────────────────────────────────
step_enable_uart() {
    echo ""
    info "${BOLD}Step 2/6:${NC} Enabling UART on GPIO 14/15..."

    local config_file=""
    for f in /boot/firmware/config.txt /boot/config.txt; do
        [[ -f "$f" ]] && config_file="$f" && break
    done

    if [[ -z "$config_file" ]]; then
        err "Cannot find /boot/config.txt or /boot/firmware/config.txt"
        exit 1
    fi

    info "  Config file: $config_file"

    # Enable UART (if not already set)
    if grep -qx 'enable_uart=1' "$config_file" 2>/dev/null; then
        ok "  UART already enabled."
    else
        echo 'enable_uart=1' | sudo tee -a "$config_file" >/dev/null
        ok "  UART enabled (added enable_uart=1 to $config_file)"
    fi

    # Disable Bluetooth if present (frees UART for Pi 3/Zero W)
    if grep -qx 'dtoverlay=disable-bt' "$config_file" 2>/dev/null; then
        ok "  Bluetooth already disabled."
    else
        echo 'dtoverlay=disable-bt' | sudo tee -a "$config_file" >/dev/null
        ok "  Bluetooth disabled (frees UART)."
    fi

    # Disable console on serial (so the PN532 can use it)
    if grep -q 'console=serial0' /boot/cmdline.txt 2>/dev/null; then
        sudo sed -i 's/console=serial0,[0-9]* //g' /boot/cmdline.txt
        ok "  Removed console from serial0 in cmdline.txt"
    fi

    # Disable serial getty service
    if systemctl is-active serial-getty@ttyAMA0.service &>/dev/null; then
        sudo systemctl disable serial-getty@ttyAMA0.service
        sudo systemctl mask serial-getty@ttyAMA0.service
        ok "  Disabled serial getty on ttyAMA0"
    fi
}

# ── Step 3: Add user to dialout group ─────────────────────────────────────
step_dialout() {
    echo ""
    info "${BOLD}Step 3/6:${NC} Adding user to dialout group..."

    if id -nG "$USER" | grep -qw 'dialout'; then
        ok "  User $USER already in dialout group."
    else
        sudo usermod -a -G dialout "$USER"
        warn "  Added $USER to dialout group. You may need to log out & back in for this to take effect."
    fi
}

# ── Step 4: Install Rust ──────────────────────────────────────────────────
step_rust() {
    echo ""
    info "${BOLD}Step 4/6:${NC} Installing Rust..."

    if command -v rustc &>/dev/null; then
        ok "  Rust already installed: $(rustc --version)"
    else
        info "  Installing Rust via rustup (this may take a while on Pi Zero)..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        ok "  Rust installed: $(rustc --version)"
    fi

    # Ensure cargo is in PATH for the rest of the script
    export PATH="$HOME/.cargo/bin:$PATH"
}

# ── Step 5: Clone / pull repo & build ─────────────────────────────────────
step_build() {
    echo ""
    info "${BOLD}Step 5/6:${NC} Building the NearField reader..."

    if [[ -d "$REPO_DIR" ]]; then
        info "  Repo exists at $REPO_DIR — pulling latest..."
        cd "$REPO_DIR"
        git pull --rebase
    else
        info "  Cloning repo..."
        git clone "$REPO_URL" "$REPO_DIR"
        cd "$REPO_DIR"
    fi

    info "  Building release binary (this may take 10-15 min on Pi Zero)..."
    cargo build --release

    # Copy binary to home for easy access
    cp "target/release/$BIN_NAME" "$HOME/nearfield"
    ok "  Binary built and copied to $HOME/nearfield"
}

# ── Step 6: Install systemd service (optional) ────────────────────────────
step_service() {
    echo ""
    info "${BOLD}Step 6/6:${NC} Installing systemd service..."

    local service_file="/etc/systemd/system/$SERVICE_NAME"
    if [[ -f "$service_file" ]]; then
        ok "  Service already installed at $service_file"
    else
        sudo tee "$service_file" >/dev/null <<SERVICEEOF
[Unit]
Description=NearField NFC Tag Reader
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME
ExecStart=$HOME/nearfield
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=NEARFIELD_DISPLAY_TYPE=stdout
Environment=NEARFIELD_SERIAL_PORT=/dev/ttyAMA0

[Install]
WantedBy=multi-user.target
SERVICEEOF
        ok "  Service file created at $service_file"
    fi

    sudo systemctl daemon-reload
    sudo systemctl enable "$SERVICE_NAME"
    sudo systemctl start "$SERVICE_NAME" || warn "  Service start may have failed — check with: sudo systemctl status $SERVICE_NAME"

    ok "  Service enabled and started."
    echo ""
    echo -e "  ${CYAN}View logs:${NC}  journalctl -u $SERVICE_NAME -f"
    echo -e "  ${CYAN}Status:${NC}    sudo systemctl status $SERVICE_NAME"
}

# ── Reboot reminder ───────────────────────────────────────────────────────
print_reboot_needed() {
    echo ""
    echo -e "${YELLOW}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${YELLOW}║${NC}  ${BOLD}Reboot required!${NC}                                            ${YELLOW}║${NC}"
    echo -e "${YELLOW}║${NC}  Run:  ${CYAN}sudo reboot${NC}                                              ${YELLOW}║${NC}"
    echo -e "${YELLOW}║${NC}  After reboot, test your NFC reader with:                         ${YELLOW}║${NC}"
    echo -e "${YELLOW}║${NC}    ${CYAN}$HOME/nearfield --scan${NC}                            ${YELLOW}║${NC}"
    echo -e "${YELLOW}╚══════════════════════════════════════════════════════════════╝${NC}"
}

# ── Main ──────────────────────────────────────────────────────────────────
main() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}     NearField NFC Reader — Automated Setup                  ${CYAN}║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "  ${BOLD}Pi Model:${NC}  $PI_MODEL"
    echo -e "  ${BOLD}User:${NC}      $USER"
    echo -e "  ${BOLD}Home:${NC}      $HOME"
    echo ""

    if [[ "$BUILD_ONLY" == true ]]; then
        info "Running in build-only mode (skipping system setup)..."
        step_rust
        step_build
        echo ""
        ok "Build complete! Binary at: $HOME/nearfield"
        echo "  Test it with: $HOME/nearfield --scan"
        exit 0
    fi

    step_system_packages
    step_enable_uart
    step_dialout
    step_rust
    step_build

    if [[ "$INSTALL_SERVICE" == true ]]; then
        step_service
    fi

    print_reboot_needed

    echo ""
    ok "Setup complete!"
}

main "$@"
