#!/bin/bash
# test - Detect attached STM32 Nucleo board and run HIL tests with the
# correct MCU feature and probe-rs runner chip.
#
# Usage:
#   ./test.sh                    # run integration + flash (non-destructive)
#   ./test.sh integration        # run only integration test
#   ./test.sh flash              # run only flash test
#   ./test.sh all --no-run       # compile tests only
#   ./test.sh flash --destructive
#   ./test.sh integration --board f446

set -e

TEST_NAME="all"
BOARD_OVERRIDE="auto"
NO_RUN=0
DESTRUCTIVE=0

usage() {
  cat <<'EOF'
Usage: ./test.sh [integration|flash|all] [options]

Options:
  --board auto|f413|f446  Select board (default: auto-detect)
  --no-run                Build tests but do not run
  --destructive           Enable flash-destructive feature (flash test only)
  -h, --help              Show this help
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    integration|flash|all)
      TEST_NAME="$1"
      shift
      ;;
    --board)
      BOARD_OVERRIDE="${2:-}"
      [[ -n "$BOARD_OVERRIDE" ]] || { echo "Missing value for --board"; exit 1; }
      shift 2
      ;;
    --no-run)
      NO_RUN=1
      shift
      ;;
    --destructive)
      DESTRUCTIVE=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1"
      usage
      exit 1
      ;;
  esac
done

RUST_TARGET="thumbv7em-none-eabihf"

ensure_tool() {
  local tool="$1" install="$2"
  command -v "$tool" >/dev/null && return 0
  echo "📦 '$tool' not found - installing..."
  eval "$install"
  command -v "$tool" >/dev/null || { echo "❌ install failed: $tool"; exit 1; }
}

if ! command -v arm-none-eabi-ld >/dev/null; then
  if command -v apt-get >/dev/null; then
    sudo apt-get install -y --no-install-recommends binutils-arm-none-eabi
  elif command -v brew >/dev/null; then
    brew install arm-none-eabi-binutils
  else
    echo "❌ Install arm-none-eabi-binutils manually."
    exit 1
  fi
fi

ensure_tool flip-link "cargo install flip-link"
ensure_tool probe-rs "cargo install probe-rs-tools --locked"
rustup target list --installed 2>/dev/null | grep -q "$RUST_TARGET" || rustup target add "$RUST_TARGET"

detect_board() {
  local lsusb_out idcode="" devid
  lsusb_out=$(lsusb 2>/dev/null)
  echo "$lsusb_out" | grep -qE "0483:(374b|374e)" || { echo ""; return; }

  for try_chip in STM32F413ZH STM32F446RE; do
    idcode=$(probe-rs read --chip "$try_chip" b32 0xE0042000 1 2>/dev/null | tr -d '[:space:]')
    [[ -n "$idcode" ]] && break
  done

  [[ -z "$idcode" ]] && { echo ""; return; }
  devid=$(( 0x$idcode & 0xFFF ))

  case "$devid" in
    $((0x463))) echo "f413" ;;
    $((0x421))) echo "f446" ;;
    *) echo "" ;;
  esac
}

BOARD="$BOARD_OVERRIDE"
if [[ "$BOARD" == "auto" ]]; then
  BOARD=$(detect_board)
fi

case "$BOARD" in
  f413)
    CHIP_NAME="STM32F413ZH"
    FEATURE="stm32f413"
    BOARD_LABEL="STM32F413ZH Nucleo-144"
    ;;
  f446)
    CHIP_NAME="STM32F446RE"
    FEATURE="stm32f446"
    BOARD_LABEL="STM32F446RE Nucleo-64"
    ;;
  *)
    echo "❌ Could not determine board. Use --board f413 or --board f446."
    exit 1
    ;;
esac

echo "🔍 Using board: $BOARD_LABEL"

FEATURES="$FEATURE,hdlc_fcs"
if [[ "$DESTRUCTIVE" -eq 1 ]]; then
  FEATURES="$FEATURES,flash-destructive"
fi

RUNNER="probe-rs run --chip $CHIP_NAME --catch-hardfault --catch-reset --rtt-scan-memory --log-format=oneline"

run_test() {
  local test_target="$1"
  local extra_args=()
  [[ "$NO_RUN" -eq 1 ]] && extra_args+=("--no-run")

  echo "🧪 cargo test --test $test_target --no-default-features --features $FEATURES ${extra_args[*]}"
  CARGO_TARGET_THUMBV7EM_NONE_EABIHF_RUNNER="$RUNNER" \
    cargo test --test "$test_target" --no-default-features --features "$FEATURES" "${extra_args[@]}"
}

case "$TEST_NAME" in
  integration)
    run_test integration
    ;;
  flash)
    run_test flash
    ;;
  all)
    run_test integration
    run_test flash
    ;;
esac