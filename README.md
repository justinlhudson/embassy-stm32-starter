# 🚀 Multi-MCU Embassy Framework - Embedded Rust Project

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org/)
[![Embassy](https://im   └── 📂 boards/                # 📟 Board-specific pin configurations & interrupt handlers
       ├── 📄 nucleo_f446re.rs       # Nucleo F446RE pins, interrupts & hardware init
       └── 📄 [future_board_configs] # Additional board configurationshields.io#### 📋 **Template System** (`config/templates/`)
- **Purpose**: MCU-specific configuration templates
- **Components**: Cargo.toml and config.toml variants for each MCU
- **Benefits**: Version-controlled, consistent configurations

#### 💾 **Memory Layouts** (`config/memory/`)
- **Purpose**: MCU-specific memory definitions for linker
- **Format**: GNU LD linker scripts defining Flash/RAM regions
- **Usage**: Copied to root `memory.x` by setup script

#### 📟 **Board Configurations** (`board.rs`)
- **Purpose**: Board-specific pin mappings, settings, and interrupt handlers
- **Format**: Rust modules with const definitions and interrupt handlers
- **Usage**: Copied to root `board.rs` by setup script
- **Contents**: Pin assignments, GPIO settings, hardware initialization, MCU-specific interrupts-v0.4+-blue.svg)](https://embassy.dev/)
[![STM32](https://img.shields.io/badge/STM32-Multi--Board-green.svg)](https://www.st.com/en/microcontrollers-microprocessors/stm32-32-bit-arm-cortex-mcus.html)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-yellow.svg)](LICENSE)

A modern, async embedded Rust project demonstrating real-time system capabilities using the Embassy framework on multiple STM32 microcontrollers. Features **automatic MCU configuration management**, hardware abstraction layers, task management, and comprehensive peripheral control with **single-command board switching**.

## 📋 Table of Contents

- [🎯 Features](#-features)
- [🔧 Hardware Requirements](#-hardware-requirements)
- [⚡ MCU Configuration System](#-mcu-configuration-system)
- [📁 Project Structure](#-project-structure)
- [🛠️ Setup & Installation](#️-setup--installation)
- [🚀 Building & Flashing](#-building--flashing)
- [⚙️ Board Configuration](#️-board-configuration)
- [🔄 Adding New MCUs](#-adding-new-mcus)
- [🧪 Testing](#-testing)
- [🤝 Contributing](#-contributing)
- [📄 License](#-license)

## 🎯 Features

- **Single-command board switching** and template-based configuration (`./setup.sh <board>`)
- **Async/await multitasking** with Embassy's runtime
- **Board-agnostic, modular code**: hardware abstraction, generic tasks, and easy MCU/board addition
- **Automatic config management**: memory.x, board.rs, Cargo.toml, .cargo/config.toml
- **RTT logging** and structured debug output via `defmt`
- **Watchdog, RTC, GPIO, and timer utilities**
- **Integration tests** and size-optimized release builds

## 🔧 Hardware Requirements

### Supported Boards

- **STM32 Nucleo-F446RE** (default, fully tested, USB-powered, built-in ST-LINK)
- Easy expansion: STM32F4, F7, H7, G4 series (see [Adding New MCUs](#-adding-new-mcus))

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


### Memory Layout
```
Flash (0x08000000): [████████████████████████████████] 512 KB
RAM   (0x20000000): [████████████████████████████████] 128 KB
```

### Configuration Architecture
```
config/              # Configuration management
├── templates/       # MCU-specific config templates (Cargo.toml, config.toml)
├── memory/          # MCU memory layouts (e.g., stm32f446re.x)
board.rs             # Board-specific pin config (copied by setup.sh)
```

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
│   │   ├── 📄 Cargo_nucleo_f446re.toml     # STM32F446RE dependencies
│   │   ├── 📄 config_nucleo_f446re.toml    # STM32F446RE build settings
│   │   └── 📄 [future_mcu_templates]       # Additional MCU templates
│   ├── 📂 memory/                # 💾 MCU memory layout definitions
│   │   ├── 📄 stm32f446re.x          # STM32F446RE memory map
│   │   └── 📄 [future_mcu_layouts]   # Additional MCU memory files
│   └── 📂 boards/                # 📟 Board-specific pin configurations
│       ├── 📄 nucleo_f446re.rs       # Nucleo F446RE pin mappings
│       └── 📄 [future_board_configs] # Additional board configurations
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
- `src/bin/`: Binaries (e.g., blinky.rs)

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
   # Should show: STM32F446RETx
   ```
3. **Driver Installation**: Usually automatic, but see [ST-LINK drivers](https://www.st.com/en/development-tools/st-link-v2.html) if needed

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

### Build Profiles
```bash
# Development build (debug symbols, RTT logging)
cargo build --bin blinky

# Release build (size optimized, no debug)
cargo build --release --bin blinky

# Check binary size
cargo bloat --release --bin blinky
```

### Advanced Flashing
```bash
# Flash without running
probe-rs download --chip STM32F446RE target/thumbv7em-none-eabihf/debug/blinky

# Flash and attach debugger
probe-rs attach --chip STM32F446RE

# Reset target
probe-rs reset --chip STM32F446RE
```

## ⚙️ Board Configuration

### Switching Between Boards
The project supports easy switching between different MCU configurations:

```bash
# Switch to STM32F446RE Nucleo board (default)
./setup.sh nucleo

# Future board support examples:
# ./setup.sh discovery  # STM32F407 Discovery board
# ./setup.sh bluepill   # STM32F103 Blue Pill board
# ./setup.sh custom     # Your custom board configuration
```


### Understanding Board Files
Each board configuration in `boards/` contains:

```rust
// Example: boards/nucleo_f446re.rs
use embassy_stm32::gpio::{Level, Pull};

pub struct BoardConfig;

impl BoardConfig {
    // LED configuration
    pub const LED_PIN: &'static str = "PA5";
    pub const LED_ACTIVE_LEVEL: Level = Level::High;
    
    // Button configuration  
    pub const BUTTON_PIN: &'static str = "PC13";
    pub const BUTTON_PULL: Pull = Pull::Down;
    
    // Board identification
    pub const BOARD_NAME: &'static str = "STM32F446RE Nucleo";
}
```

pub struct BoardConfig;
target = "thumbv7em-none-eabihf"

## 🧪 Testing

### Hardware-in-the-Loop Tests
```bash
# Run integration tests on hardware
cargo test --test integration

# Run specific test
cargo test --test integration -- button_test
```

### Unit Tests
```bash
# Run software-only unit tests
cargo test --lib
```

### Continuous Integration
The project includes CI-ready configurations for:
- **Build verification** across multiple targets
- **Code quality checks** (clippy, formatting)
- **Documentation generation**

## 🔧 Troubleshooting

### Common Issues

#### 1. **Probe Connection Failed**
```
Error: Probe was not found.
```
**Solution**: Check USB connection, install ST-LINK drivers, verify with `probe-rs list`

#### 2. **Build Errors**
```
error: linking with `rust-lld` failed: exit status: 1
```
**Solution**: Ensure correct target: `rustup target add thumbv7em-none-eabihf`

#### 3. **RTT Not Working**
```
No RTT data received
```
**Solution**: Verify debug build (`cargo build` not `cargo build --release`), check RTT buffer size

#### 4. **Watchdog Reset Loop**
**Solution**: Increase watchdog timeout or decrease main loop delay in `blinky.rs`

### Debug Strategies
1. **LED Patterns**: Use different blink rates to indicate system states
2. **RTT Logging**: Add `info!()`, `warn!()`, `error!()` messages for debugging
3. **Probe Debugging**: Use `probe-rs gdb` for step-through debugging
4. **Logic Analyzer**: Monitor GPIO pins for timing verification

## 🔄 Development Workflow

### Recommended Tools
- **VS Code** with rust-analyzer extension
- **probe-rs VS Code extension** for integrated debugging
- **RTT Viewer** for real-time log monitoring
- **STM32CubeMX** for peripheral reference (optional)

### Best Practices
- Commit frequently, use meaningful messages
- Run `cargo clippy` and `cargo fmt` before commits
- Test on hardware regularly
- Keep docs and code comments up to date
- Profile with `cargo bloat` for size

## 🤝 Contributing

### Development Setup
1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Configure for your development board: `./setup.sh nucleo`
4. Make your changes following the existing code style
5. Test on real hardware with your configuration
6. Submit a pull request with clear description of changes

### Code Style
- Use `cargo fmt` for formatting
- Follow Rust naming conventions
- Add comprehensive documentation for public APIs
- Include hardware testing for new features

### Contribution Areas
- 🎯 **Additional STM32 board support** - Add new MCU configurations
- 🔧 **New peripheral drivers** - SPI, I2C, UART abstractions
- 📚 **Documentation improvements** - Better examples and guides
- 🧪 **Test coverage expansion** - More hardware testing scenarios
- ⚡ **Performance optimizations** - Memory and timing improvements
- 🛠️ **Setup script enhancements** - Better error handling, validation
- 📋 **Template improvements** - More comprehensive MCU support

## 📚 Additional Resources

### Documentation
- [Embassy Framework Documentation](https://embassy.dev/)
- [STM32F446RE Reference Manual](https://www.st.com/resource/en/reference_manual/dm00135183-stm32f446xx-advanced-armbased-32bit-mcus-stmicroelectronics.pdf)
- [Rust Embedded Book](https://doc.rust-lang.org/stable/embedded-book/)
- [probe-rs Documentation](https://probe.rs/)

### Community
- [Rust Embedded Matrix Chat](https://matrix.to/#/#rust-embedded:matrix.org)
- [Embassy Matrix Chat](https://matrix.to/#/#embassy-rs:matrix.org)
- [STM32 Community Forum](https://community.st.com/s/)

## 📄 License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution Licensing
Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.

## 👤 Author

**Justin L. Hudson** - *Project Creator & Maintainer*
- Email: justinlhudson@gmail.com
- GitHub: [@justinlhudson](https://github.com/justinlhudson)

---

**Built with ❤️ using [Embassy](https://embassy.dev/), [Rust](https://www.rust-lang.org/), and ☕**

*For questions or support, please open an issue on GitHub.*

## ⚡ Quick Commands Reference

```bash
# Essential commands for this project:
./setup.sh nucleo              # Configure for STM32F446RE
cargo build --bin blinky       # Build application
cargo run --bin blinky         # Flash and run with logging
cargo check                    # Quick syntax check
probe-rs list                  # Verify hardware connection

# Development commands:
cargo clippy                   # Code quality check
cargo fmt                      # Format code
cargo test --lib              # Run unit tests  
cargo test --test integration # Run hardware tests
```