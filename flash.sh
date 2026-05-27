#!/bin/bash
# flash - Detect attached STM32 Nucleo board, build with the right features,
# flash, and stream RTT logs.
#
# Usage: ./flash.sh [binary]      (binary defaults to "example")

set -e

BINARY="${1:-example}"

# ── Pre-flight: required tools (idempotent) ───────────────────────────────
RUST_TARGET="thumbv7em-none-eabihf"

ensure_tool() {
  local tool="$1" install="$2"
  command -v "$tool" >/dev/null && return 0
  echo "📦 '$tool' not found – installing..."
  eval "$install"
  command -v "$tool" >/dev/null || { echo "❌ install failed: $tool"; exit 1; }
}

if ! command -v arm-none-eabi-ld >/dev/null; then
  if   command -v apt-get >/dev/null; then sudo apt-get install -y --no-install-recommends binutils-arm-none-eabi
  elif command -v brew    >/dev/null; then brew install arm-none-eabi-binutils
  else echo "❌ Install arm-none-eabi-binutils manually."; exit 1
  fi
fi
ensure_tool flip-link "cargo install flip-link"
ensure_tool probe-rs  "cargo install probe-rs-tools --locked"
rustup target list --installed 2>/dev/null | grep -q "$RUST_TARGET" \
  || rustup target add "$RUST_TARGET"

# ── 1. Detect board via DBGMCU_IDCODE ─────────────────────────────────────
# Both Nucleo-64 (F446RE) and Nucleo-144 (F413ZH) ship with ST-Link/V2.1
# (0483:374b), so USB VID/PID isn't enough. Read the device ID instead.
detect_board() {
  local lsusb_out
  lsusb_out=$(lsusb 2>/dev/null)
  echo "$lsusb_out" | grep -qE "0483:(374b|374e)" || { echo ""; return; }
  # Read DBGMCU_IDCODE. probe-rs needs *some* chip name; try the F4 variants
  # we know about until one attaches successfully.
  local idcode=""
  for try_chip in STM32F413ZH STM32F446RE; do
    idcode=$(probe-rs read --chip "$try_chip" b32 0xE0042000 1 2>/dev/null | tr -d '[:space:]')
    [[ -n "$idcode" ]] && break
  done
  [[ -z "$idcode" ]] && { echo ""; return; }
  local devid=$(( 0x$idcode & 0xFFF ))
  case "$devid" in
    $((0x421))) echo "f446" ;;
    $((0x463))) echo "f413" ;;
    *)          echo "" ;;
  esac
}

BOARD=$(detect_board)
if [[ -z "$BOARD" ]]; then
  echo "❌ No supported STM32 Nucleo board detected."
  echo "   Supported: Nucleo-64 F446RE (devid 0x421), Nucleo-144 F413ZH (devid 0x463)"
  exit 1
fi

case "$BOARD" in
  f446) BOARD_LABEL="STM32F446RE Nucleo-64";  CHIP_NAME="STM32F446RE"; FEATURE="stm32f446" ;;
  f413) BOARD_LABEL="STM32F413ZH Nucleo-144"; CHIP_NAME="STM32F413ZH"; FEATURE="stm32f413" ;;
esac

echo "🔍 Detected: $BOARD_LABEL  (feature: $FEATURE)"

# ── 2. Build ──────────────────────────────────────────────────────────────
echo "🔨 Building '$BINARY' --features $FEATURE,hdlc_fcs"
cargo build --bin "$BINARY" --no-default-features --features "$FEATURE,hdlc_fcs"

ELF="target/$RUST_TARGET/debug/$BINARY"
[[ -f "$ELF" ]] || { echo "❌ Build artifact not found: $ELF"; exit 1; }

# ── 3. Flash & stream logs ────────────────────────────────────────────────
trap 'echo ""; echo "👋 Log stream stopped."; exit 0' INT TERM

echo "⚡ Flashing '$BINARY' to $CHIP_NAME (Ctrl+C to stop)..."
probe-rs run \
  --chip "$CHIP_NAME" \
  --catch-hardfault \
  --catch-reset \
  --rtt-scan-memory \
  --log-format "{t} [{L}] {s}" \
  "$ELF" || true
