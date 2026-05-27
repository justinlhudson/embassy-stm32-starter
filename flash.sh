#!/bin/bash

# flash - Detect attached STM32 Nucleo board, run setup, build, and flash.
# Usage: ./flash [binary]  (defaults to "example")

set -e

BINARY="${1:-example}"

# ── 1. Detect board by reading DBGMCU_IDCODE via probe-rs ────────────────────
#
# USB VID/PID alone is not reliable: both Nucleo-64 (F446RE) and Nucleo-144
# (F413ZH) ship with ST-Link/V2.1 (0483:374b). Read the chip's DBGMCU_IDCODE
# register at 0xE0042000 instead — the lower 12 bits are the device ID:
#   0x421 → STM32F446
#   0x463 → STM32F413/F423

detect_board() {
    # First confirm an ST-Link is present at all
    local lsusb_out
    lsusb_out=$(lsusb 2>/dev/null)
    if ! echo "$lsusb_out" | grep -qE "0483:(374b|374e)"; then
        echo ""
        return
    fi

    # --chip just needs to be any ARM Cortex-M STM32 to attach SWD; the
    # IDCODE read works regardless of whether it matches the real silicon.
    local idcode
    idcode=$(probe-rs read --chip STM32F446RE b32 0xE0042000 1 2>/dev/null | tr -d '[:space:]')
    if [[ -z "$idcode" ]]; then
        echo ""
        return
    fi

    # Mask to lower 12 bits (device ID)
    local devid=$(( 0x$idcode & 0xFFF ))
    case "$devid" in
        $((0x421))) echo "nucleo" ;;     # STM32F446
        $((0x463))) echo "nucleo144" ;;  # STM32F413/F423
        *)          echo "" ;;
    esac
}

BOARD=$(detect_board)

if [[ -z "$BOARD" ]]; then
    echo "❌ No supported STM32 Nucleo board detected on USB."
    echo "   Make sure the board is plugged in with a data-capable cable."
    echo "   Supported: Nucleo-64 F446RE (devid 0x421), Nucleo-144 F413ZH (devid 0x463)"
    exit 1
fi

case "$BOARD" in
    nucleo)     BOARD_LABEL="STM32F446RE Nucleo-64";  CHIP_NAME="STM32F446RE" ;;
    nucleo144)  BOARD_LABEL="STM32F413ZH Nucleo-144"; CHIP_NAME="STM32F413ZH" ;;
esac

echo "🔍 Detected: $BOARD_LABEL"

# ── 2. Run setup ──────────────────────────────────────────────────────────────

echo "⚙️  Running setup for '$BOARD'..."
./setup.sh "$BOARD"

# ── 3. Build ──────────────────────────────────────────────────────────────────

echo "🔨 Building '$BINARY'..."
cargo build --bin "$BINARY"

# Resolve the ELF path from the cargo target directory
TARGET_TRIPLE="thumbv7em-none-eabihf"
ELF="target/$TARGET_TRIPLE/debug/$BINARY"

if [[ ! -f "$ELF" ]]; then
    echo "❌ Build artifact not found: $ELF"
    exit 1
fi

# ── 4. Flash & stream logs ────────────────────────────────────────────────────

# Ctrl+C exits cleanly (probe-rs exits with SIGINT; suppress the non-zero status)
trap 'echo ""; echo "👋 Log stream stopped."; exit 0' INT TERM

echo "⚡ Flashing and streaming logs from '$BINARY' (Ctrl+C to stop)..."
probe-rs run \
    --chip "$CHIP_NAME" \
    --catch-hardfault \
    --catch-reset \
    --rtt-scan-memory \
    --log-format "{t} [{L}] {s}" \
    "$ELF" || true
