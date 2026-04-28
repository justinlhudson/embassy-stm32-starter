#!/bin/bash

# Board Configuration Setup Script
# Run this manually when switching between different MCU/board configurations
# 
## Usage: ./setup [board]
# Available boards: nucleo

set -e

BOARD="${1:-nucleo}"

# ── Pre-flight: verify / install required tools ───────────────────────────────

RUST_TARGET="thumbv7em-none-eabihf"

check_and_install() {
    local tool="$1"
    local install_cmd="$2"
    if ! command -v "$tool" &>/dev/null; then
        echo "📦 '$tool' not found – installing..."
        eval "$install_cmd"
        if ! command -v "$tool" &>/dev/null; then
            echo "❌ Failed to install '$tool'. Please install it manually and re-run."
            exit 1
        fi
        echo "✅ '$tool' installed."
    fi
}

# arm-none-eabi-ld  (system package — binutils only, no GCC/newlib needed)
if ! command -v arm-none-eabi-ld &>/dev/null; then
    echo "📦 'arm-none-eabi-ld' not found – installing via apt..."
    if command -v apt-get &>/dev/null; then
        sudo apt-get install -y --no-install-recommends binutils-arm-none-eabi
    elif command -v brew &>/dev/null; then
        brew install arm-none-eabi-binutils
    else
        echo "❌ Cannot install 'arm-none-eabi-ld': no supported package manager found."
        echo "   Install it manually (apt: binutils-arm-none-eabi)."
        exit 1
    fi
fi

# flip-link  (cargo install)
check_and_install flip-link "cargo install flip-link"

# probe-rs  (cargo install)
check_and_install probe-rs "cargo install probe-rs-tools --locked"

# Rust cross-compilation target
if ! rustup target list --installed 2>/dev/null | grep -q "$RUST_TARGET"; then
    echo "📦 Rust target '$RUST_TARGET' not installed – adding..."
    rustup target add "$RUST_TARGET"
fi

echo "✅ All dependencies satisfied."

# ─────────────────────────────────────────────────────────────────────────────

# Function to get current memory configuration
get_current_memory_target() {
    if [[ -f "memory.x" ]]; then
        if grep -q "512K" memory.x && grep -q "128K" memory.x; then
            echo "STM32F446RE (512K flash, 128K RAM)"
        elif grep -q "1536K" memory.x && grep -q "256K" memory.x; then
            echo "STM32F413ZH (1536K flash, 256K RAM)"
        else
            echo "Unknown configuration"
        fi
    else
        echo "No memory.x found"
    fi
}

if [[ "$BOARD" == "help" || "$BOARD" == "-h" || "$BOARD" == "--help" ]]; then
    echo "🔧 Board Configuration Setup"
    echo ""
    echo "Usage: ./setup.sh [board]"
    echo ""
    echo "Available boards:"
    echo "  nucleo        - STM32F446RE Nucleo board (default)"
    echo "  nucleo144     - STM32F413ZH Nucleo-144 board"
    echo ""
    echo "Current memory.x points to: $(get_current_memory_target)"
    exit 0
fi

# Configure based on board selection
case "$BOARD" in
    "nucleo"|"nucleo-f446re")
        MCU_NAME="STM32F446RE"
        BOARD_TYPE="Nucleo"
        BOARD_CONFIG_FILE="nucleo_f446re.rs"
        STM32_FAMILY="stm32f446"
        STM32_MCU="stm32f446re"
        ;;
    "nucleo144"|"nucleo-144"|"nucleo144-f413zh")
        MCU_NAME="STM32F413ZH"
        BOARD_TYPE="Nucleo-144"
        BOARD_CONFIG_FILE="nucleo144_f413zh.rs"
        STM32_FAMILY="stm32f413"
        STM32_MCU="stm32f413zh"
        ;;
    *)
        echo "❌ Unknown board: $BOARD"
        echo "Available boards: nucleo, nucleo144"
        exit 1
        ;;
esac

# Derive other variables from core settings
CHIP_NAME="$MCU_NAME"                                    # Same as MCU name
BOARD_NAME="$MCU_NAME $BOARD_TYPE board"                 # "STM32F446RE Nucleo board"
BOARD_DESCRIPTION="$MCU_NAME $BOARD_TYPE board"          # Same as board name
BOARD_CONFIG="src/board/$BOARD_CONFIG_FILE"              # Full path to config file

# Function to substitute template variables
substitute_template() {
    local template_file="$1"
    local output_file="$2"
    
    if [[ ! -f "$template_file" ]]; then
        echo "❌ Template file not found: $template_file"
        return 1
    fi
    
    # Use sed to substitute all template variables
    sed -e "s/{{BOARD_DESCRIPTION}}/$BOARD_DESCRIPTION/g" \
        -e "s/{{CHIP_NAME}}/$CHIP_NAME/g" \
        -e "s/{{BOARD_CONFIG_FILE}}/$BOARD_CONFIG_FILE/g" \
        -e "s/{{STM32_FAMILY}}/$STM32_FAMILY/g" \
        -e "s/{{STM32_MCU}}/$STM32_MCU/g" \
        "$template_file" > "$output_file"
    
    return 0
}

echo "🎯 Configuring for $BOARD_NAME ($MCU_NAME)"

# Remove old config/memory file logic and variable
# Remove old config/memory files if they exist (cleanup)
rm -f config/memory/stm32f413zh.x config/memory/stm32f446re.x

# Copy memory.template.x and only uncomment the correct MEMORY block, never the board name
MEMORY_TEMPLATE="memory.template.x"
MEMORY_FILE="memory.x"
if [[ -f "$MEMORY_TEMPLATE" ]]; then
    cp "$MEMORY_TEMPLATE" "$MEMORY_FILE"
    if [[ "$BOARD" == "nucleo" || "$BOARD" == "nucleo-f446re" ]]; then
        awk '/\/\* STM32F446RE \(Nucleo-64\) \*\// {print; inblock=1; next} /\/\* STM32F413ZH \(Nucleo-144\) \*\// {inblock=0} inblock && /^\s*\/\*/ {sub(/^\s*\/\*/, ""); sub(/\*\/$/, ""); print; next} inblock && /\*\// {sub(/\*\//, ""); print; inblock=0; next} {print}' "$MEMORY_FILE" > "$MEMORY_FILE.tmp" && mv "$MEMORY_FILE.tmp" "$MEMORY_FILE"
    elif [[ "$BOARD" == "nucleo144" || "$BOARD" == "nucleo-144" || "$BOARD" == "nucleo144-f413zh" ]]; then
        awk '/\/\* STM32F413ZH \(Nucleo-144\) \*\// {print; inblock=1; next} /\/\* STM32F446RE \(Nucleo-64\) \*\// {inblock=0} inblock && /^\s*\/\*/ {sub(/^\s*\/\*/, ""); sub(/\*\/$/, ""); print; next} inblock && /\*\// {sub(/\*\//, ""); print; inblock=0; next} {print}' "$MEMORY_FILE" > "$MEMORY_FILE.tmp" && mv "$MEMORY_FILE.tmp" "$MEMORY_FILE"
    fi
    echo "✅ Updated $MEMORY_FILE for $BOARD"
else
    echo "❌ $MEMORY_TEMPLATE not found."
    exit 1
fi

# Generate board.rs from template
if substitute_template "board.template.rs" "board.rs"; then
    echo "✅ Generated board.rs from template"
else
    echo "❌ Failed to generate board.rs from template"
    exit 1
fi

# Generate Cargo.toml from template
if substitute_template "Cargo.template.toml" "Cargo.toml"; then
    echo "✅ Generated Cargo.toml from template"
else
    echo "❌ Failed to generate Cargo.toml from template"
    exit 1
fi

# Generate .cargo/config.toml from template
mkdir -p .cargo
if substitute_template ".cargo/config.template.toml" ".cargo/config.toml"; then
    echo "✅ Generated .cargo/config.toml from template"
else
    echo "❌ Failed to generate .cargo/config.toml from template"
    exit 1
fi

# Always copy the launch template before replacing the chip field
LAUNCH_TEMPLATE=".vscode/launch.template.json"
LAUNCH_FILE=".vscode/launch.json"
if [[ -f "$LAUNCH_TEMPLATE" ]]; then
    cp "$LAUNCH_TEMPLATE" "$LAUNCH_FILE"
    # Simple, portable sed replacement for chip field
    sed -i "s/\"chip\": \"{{CHIP_NAME}}\".*/\"chip\": \"$CHIP_NAME\",/" "$LAUNCH_FILE"
    echo "✅ Updated $LAUNCH_FILE chip field to $CHIP_NAME"
else
    echo "⚠️  $LAUNCH_TEMPLATE not found; skipping chip update."
fi

echo "🔨 Ready to build with: cargo build --bin example"
