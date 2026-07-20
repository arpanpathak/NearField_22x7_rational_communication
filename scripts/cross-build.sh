#!/usr/bin/env bash
#
# cross-build.sh — Cross-compile the NearField reader for Pi and optionally copy it over.
#
# Usage:
#   ./scripts/cross-build.sh                              # Just build
#   ./scripts/cross-build.sh --deploy pi@raspberrypi.local  # Build + scp to Pi
#
# Requirements:
#   - Docker (cross builds in a containerized ARM toolchain)
#   - cargo install cross
#   - rustup target add armv7-unknown-linux-gnueabihf

set -euo pipefail

RED='\033[0;31m';  GREEN='\033[0;32m';  YELLOW='\033[1;33m'
CYAN='\033[0;36m'; BOLD='\033[1m';       NC='\033[0m'
info()  { echo -e "${CYAN}[INFO]${NC}  $*"; }
ok()    { echo -e "${GREEN}[OK]${NC}    $*"; }
err()   { echo -e "${RED}[ERR]${NC}   $*" >&2; }

TARGET="armv7-unknown-linux-gnueabihf"
BIN_NAME="nearfield_22x7_rational_communication"
DEPLOY_TARGET=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --deploy) DEPLOY_TARGET="$2"; shift 2 ;;
        --help|-h)
            echo "Usage: $0 [--deploy user@host]"
            echo ""
            echo "  --deploy user@host    Build + scp binary to Pi"
            echo "  --help                Show this help"
            exit 0
            ;;
        *) err "Unknown option: $1"; exit 1 ;;
    esac
done

info "Cross-compiling for $TARGET..."

# Make sure cross is installed
if ! command -v cross &>/dev/null; then
    err "'cross' is not installed. Run: cargo install cross --git https://github.com/cross-rs/cross"
    exit 1
fi

cross build --release --target "$TARGET"

BINARY="target/$TARGET/release/$BIN_NAME"

if [[ ! -f "$BINARY" ]]; then
    err "Build failed — binary not found at $BINARY"
    exit 1
fi

ok "Binary built: $BINARY ($(du -h "$BINARY" | cut -f1))"

if [[ -n "$DEPLOY_TARGET" ]]; then
    info "Copying to $DEPLOY_TARGET:~/nearfield ..."
    scp "$BINARY" "$DEPLOY_TARGET:~/nearfield"
    ok "Deployed! SSH into the Pi and run: ~/nearfield --scan"
fi
