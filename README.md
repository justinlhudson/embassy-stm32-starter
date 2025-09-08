
# 🚀 Multi-MCU Embassy Framework - Embedded Rust Project

A modern, async embedded Rust project demonstrating real-time system capabilities using the Embassy framework on multiple STM32 microcontrollers. Features **automatic MCU configuration management**, hardware abstraction layers, task management, and comprehensive peripheral control with **single-command board switching**.

## 📋 Table of Contents

- [Hardware Requirements](#-hardware-requirements)
- [MCU Configuration System](#-mcu-configuration-system)
- [Project Structure](#-project-structure)
- [Setup & Installation](#️-setup--installation)
- [Building & Flashing](#-building--flashing)
- [Testing](#-testing)
- [License](#-license)

## 🔧 Hardware Requirements

### Supported Boards

- **STM32 Nucleo-F446RE** (default, fully tested, USB-powered, built-in ST-LINK)
- Easy expansion: STM32F4, F7, H7, G4 series

**Board Components:**

| Component | Pin | Function | Description |
|-----------|-----|----------|-------------|
| **User LED (LD2)** | `PA5` | Status indicator | Green LED |
| **User Button (B1)** | `PC13` | Input control | Blue button |
| **ST-LINK** | USB | Debug interface | Programming, RTT, debug |

## ⚡ MCU Configuration System

### Automatic Configuration Management
The project uses a **template-based configuration system** that automatically manages all MCU-specific settings with a single command.

#### **Quick Board Switch** 🚀
```bash
# Switch to STM32F446RE Nucleo board
./setup.sh nucleo

# Output:
# ✅ Updated memory.x
# ✅ Updated board.rs  
# ✅ Updated Cargo.toml
# ✅ Updated .cargo/config.toml
```

#### **What Gets Configured**
The setup script automatically updates **4 critical files**:

| File | Purpose | Contents |
|------|---------|----------|
| `memory.x` | Linker memory layout | Flash/RAM sizes and addresses |
| `board.rs` | Board-specific pins | LED pins, button pins, configurations |
| `Cargo.toml` | MCU dependencies | Embassy features, chip-specific crates |
| `.cargo/config.toml` | Build configuration | Target, runner, debug settings |



### Configuration Architecture
```
config/              # Configuration management
├── templates/       # MCU-specific config templates (Cargo.toml, config.toml)
├── memory/          # MCU memory layouts (e.g., stm32f446re.x)
board.rs             # Board-specific pin config (copied by setup.sh)
```


## 🖥️ VS Code Support

This project includes a pre-configured `.vscode` directory for Visual Studio Code, providing recommended settings and launch configurations for embedded development.

### Debugging
The included VS Code setup provides launch configurations for debugging embedded targets using `probe-rs` and RTT logging. Simply open the project in VS Code, connect your board, and use the Run & Debug panel to start a debug session.

## 📁 Project Structure

```
embassy_stm32_starter/
├── 📄 setup.sh                   # 🎯 MCU configuration management script
├── 📄 Cargo.toml                 # 🔄 Active project configuration (managed by setup.sh)
├── 📄 memory.x                   # 🔄 Active memory layout (managed by setup.sh) 
├── 📄 board.rs                   # 🔄 Active board configuration (managed by setup.sh)
├──  .cargo/
│   └── 📄 config.toml            # 🔄 Active build settings (managed by setup.sh)
├── 📂 config/                    # 📂 Configuration management directory
│   ├── 📂 templates/             # 📋 MCU-specific configuration templates
│   │   ├── 📄 Cargo.template.toml        # Cargo.toml template
│   │   ├── 📄 config.template.toml       # config.toml template
│   │   └── 📄 board.template.rs          # Board config template
│   ├── 📂 memory/                # 💾 MCU memory layout definitions
│   │   ├── 📄 stm32f413zh.x           # STM32F413ZH memory map
│   │   ├── 📄 stm32f446re.x           # STM32F446RE memory map
│   # (Board-specific pin config is copied to board.rs by setup.sh)
├── 📂 src/
│   ├── 📄 lib.rs                 # Library root & inline module declarations
│   ├── 📂 hardware/              # Hardware Abstraction Layer (HAL)
│   │   ├── 📄 gpio.rs            # GPIO utilities & generic board configs
│   │   └── 📄 timers.rs          # Timing constants & delay functions
│   ├── 📂 common/                # 🔄 Application Layer
│   │   └── 📄 tasks.rs           # Embassy async task definitions
│   └── 📂 bin/                   # Binary applications
│       └── 📄 blinky.rs          # 🎯 Main Embassy async application (MCU-agnostic)
└── 📂 tests/                     # Integration tests
    └── 📄 integration.rs         # Hardware testing
```

## 🛠️ Setup & Installation

### Prerequisites
1. **Rust Toolchain** (1.70.0 or later)
   ```bash
   # Install Rust
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   
   # Add ARM Cortex-M target
   rustup target add thumbv7em-none-eabihf
   ```

2. **Probe-rs** (Hardware interface tool)
   ```bash
   # Install probe-rs for flashing and debugging
   cargo install probe-rs-tools --features cli
   
   # Verify installation
   probe-rs list
   ```

3. **ARM GCC Toolchain** (For some build dependencies)
   ```bash
   # macOS
   brew install arm-none-eabi-gcc
   
   # Ubuntu/Debian
   sudo apt install gcc-arm-none-eabi
   
   # Arch Linux
   sudo pacman -S arm-none-eabi-gcc
   ```

### Hardware Setup
1. **Connect Board**: USB cable to ST-LINK connector
2. **Verify Connection**: 
   ```bash
   probe-rs list
   ```

## 🚀 Building & Flashing

### Quick Start
```bash
# Clone and navigate to project
git clone <repository-url>
cd embassy_stm32_starter

# Configure for STM32F446RE Nucleo board
./setup.sh nucleo

# Build the project
cargo build --bin blinky

# Flash and run with RTT logging
cargo run --bin blinky
```

## 🧪 Testing

### Hardware-in-the-Loop Tests
```bash
# Run integration tests on hardware
cargo test --test integration

# Run specific test
cargo test --test integration -- button_test
```

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


## 👤 Author

**Justin L. Hudson** - *Project Creator & Maintainer*
- Email: justinlhudson@gmail.com
- GitHub: [@justinlhudson](https://github.com/justinlhudson)

---