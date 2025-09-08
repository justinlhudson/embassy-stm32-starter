#!/bin/bash

# Board Configuration Setup Script
# Run this manually when switching between different MCU/board configurations
# 
# Usage: ./setup.sh <board>
# Available boards: nucleo

set -e

BOARD="${1:-}"

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

# Show help if no arguments
if [[ -z "$BOARD" ]]; then
    echo "🔧 Board Configuration Setup"
    echo ""
    echo "Usage: ./setup.sh <board>"
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
        MEMORY_LAYOUT="config/memory/stm32f446re.x"
        BOARD_CONFIG="src/boards/nucleo_f446re.rs"
        BOARD_DESCRIPTION="STM32F446RE Nucleo board"
        BOARD_NAME="STM32F446RE Nucleo board"
        MCU_NAME="STM32F446RE"
        CHIP_NAME="STM32F446RE"
        BOARD_CONFIG_FILE="nucleo_f446re.rs"
        STM32_HAL_FEATURE="stm32f446"
        STM32_EMBASSY_FEATURE="stm32f446re"
        ;;
    "nucleo144"|"nucleo-144"|"nucleo144-f413zh")
        MEMORY_LAYOUT="config/memory/stm32f413zh.x"
        BOARD_CONFIG="src/boards/nucleo144_f413zh.rs"
        BOARD_DESCRIPTION="STM32F413ZH Nucleo-144 board"
        BOARD_NAME="STM32F413ZH Nucleo-144 board"
        MCU_NAME="STM32F413ZH"
        CHIP_NAME="STM32F413ZH"
        BOARD_CONFIG_FILE="nucleo144_f413zh.rs"
        STM32_HAL_FEATURE="stm32f413"
        STM32_EMBASSY_FEATURE="stm32f413zh"
        ;;
    *)
        echo "❌ Unknown board: $BOARD"
        echo "Available boards: nucleo, nucleo144"
        exit 1
        ;;
esac

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
        -e "s/{{STM32_HAL_FEATURE}}/$STM32_HAL_FEATURE/g" \
        -e "s/{{STM32_EMBASSY_FEATURE}}/$STM32_EMBASSY_FEATURE/g" \
        "$template_file" > "$output_file"
    
    return 0
}

echo "🎯 Configuring for $BOARD_NAME ($MCU_NAME)"

# Copy the appropriate memory layout to memory.x
if [[ -f "$MEMORY_LAYOUT" ]]; then
    cp "$MEMORY_LAYOUT" "memory.x"
    echo "✅ Updated memory.x from $MEMORY_LAYOUT"
else
    echo "❌ Memory layout file not found: $MEMORY_LAYOUT"
    exit 1
fi

# Generate board.rs from template
if substitute_template "config/templates/board.template.rs" "board.rs"; then
    echo "✅ Generated board.rs from template"
else
    echo "❌ Failed to generate board.rs from template"
    exit 1
fi

# Generate Cargo.toml from template
if substitute_template "config/templates/Cargo.template.toml" "Cargo.toml"; then
    echo "✅ Generated Cargo.toml from template"
else
    echo "❌ Failed to generate Cargo.toml from template"
    exit 1
fi

# Generate .cargo/config.toml from template
if substitute_template "config/templates/config.template.toml" ".cargo/config.toml"; then
    echo "✅ Generated .cargo/config.toml from template"
else
    echo "❌ Failed to generate .cargo/config.toml from template"
    exit 1
fi

echo "🔨 Ready to build with: cargo build --bin blinky"
