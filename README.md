# 🚀 Embassy STM32 Starter

> **✨ Magic Setup:** Instantly switch between supported STM32 boards and auto-configure your project with a single command: `./setup <board>`. The setup script updates all configs, linker scripts, and VS Code debug settings for you!

A modern async embedded Rust project template using the **Embassy framework** for STM32 microcontrollers. Features **automatic multi-board configuration**, HDLC communication, comprehensive hardware abstraction, and **automatic crash recovery**.

## ✨ Features

- 🎯 **Multi-Board Support**: STM32F446RE (Nucleo-64) and STM32F413ZH (Nucleo-144)
- 🔄 **One-Command Setup**: Automatic board configuration with `./setup nucleo`
- 📡 **HDLC Communication**: Reliable serial protocol with optional CRC-16
- 💾 **Flash Storage**: Direct register access with embassy-stm32 v0.4.0 bug workarounds
- 🛡️ **Auto-Recovery**: Hardware watchdog + hardfault auto-reset for crash protection
- ⚡ **Async Tasks**: LED, button, RTC, and communication handling
- 🏗️ **Conditional Compilation**: MCU-specific features via cargo flags
- 🔧 **VS Code Ready**: Pre-configured debugging and IntelliSense
- ✅ **Hardware Testing**: Integration tests on real hardware

## 🏗️ Architecture & Configuration

[note]

## 🧩 Heapless Design

This project uses the [`heapless`](https://docs.rs/heapless) crate for all dynamic data structures, such as `heapless::Vec`. This means:

- **No dynamic heap allocation**: All collections have fixed, compile-time capacities and are allocated on the stack or in static memory.
- **Deterministic memory usage**: Maximum RAM usage is known at compile time, with no risk of heap fragmentation or allocation failures at runtime.
- **Embedded-friendly**: Suitable for microcontrollers and real-time systems where dynamic allocation is discouraged or unavailable.

As a result, reported heap usage is always zero unless a dynamic allocator is added. All memory usage comes from stack and statically allocated buffers.

### Supported Boards

| Board          | MCU         | Flash  | RAM   | Serial | LED | Button | Flash Storage  |
| -------------- | ----------- | ------ | ----- | ------ | --- | ------ | -------------- |
| **Nucleo-64**  | STM32F446RE | 512KB  | 128KB | USART2 | PA5 | PC13   | Sector (128KB) |
| **Nucleo-144** | STM32F413ZH | 1536KB | 320KB | USART3 | PB0 | PC13   | Sector (128KB) |

### Quick Setup

```bash
# Configure for your board
./setup nucleo          # STM32F446RE (default)
./setup nucleo144       # STM32F413ZH

# Build and run
cargo run --bin example
```

The setup script automatically configures `Cargo.toml`, `memory.x`, `board.rs`, `.cargo/config.toml`, and VS Code debugging for the selected board with **conditional compilation support**.

### Core Components

- **Embassy Framework**: Async/await runtime for embedded systems with v0.4.0 compatibility
- **HDLC Protocol**: Reliable serial communication with frame detection and CRC-16
- **Flash Storage**: Direct STM32F4 register access bypassing embassy flash bugs
- **Auto-Recovery**: Hardware watchdog (1s timeout) + hardfault reset for crash protection
- **Task Management**: LED blinking, button monitoring, RTC clock, communication handling
- **Conditional Compilation**: MCU family features (`stm32f446`, `stm32f413`) for targeted builds
- **Template System**: Automated configuration for different MCU targets
- **Hardware Abstraction**: Board-agnostic interfaces for common peripherals

## ⚡ Configuration

### Automatic Configuration Management

The project uses a **template-based configuration system** that automatically manages all MCU-specific settings with a single command.

### Supported Boards

| Board                 | MCU         | Flash  | RAM   | Serial           | LED         | Button |
| --------------------- | ----------- | ------ | ----- | ---------------- | ----------- | ------ |
| **Nucleo-64 F446RE**  | STM32F446RE | 512KB  | 128KB | USART2 (PA2/PA3) | PA5 (Green) | PC13   |
| **Nucleo-144 F413ZH** | STM32F413ZH | 1536KB | 320KB | USART3 (PD8/PD9) | PB0 (Green) | PC13   |

### Automatic Configuration Management

The project uses a **template-based configuration system** that automatically manages all MCU-specific settings with a single command.

#### **Quick Board Switch** 🚀

```bash
# Configure for STM32F446RE Nucleo board (default)
./setup nucleo

# Configure for STM32F413ZH Nucleo-144 board
./setup nucleo144

# Show help and available options
./setup --help
```

#### **What Gets Configured**

The setup script automatically updates **5 critical files**:

| File                  | Purpose                 | Contents                                          |
| --------------------- | ----------------------- | ------------------------------------------------- |
| `Cargo.toml`          | Dependencies & features | MCU-specific Embassy features, HAL crate features |
| `memory.x`            | Linker memory layout    | Flash/RAM sizes and addresses for target MCU      |
| `board.rs`            | Active board config     | Re-exports the correct board implementation       |
| `.cargo/config.toml`  | Build configuration     | Target, runner, linker flags for target MCU       |
| `.vscode/launch.json` | VS Code debug config    | Probe-rs chip configuration for debugging         |

## � Development

### VS Code Integration

The project includes comprehensive VS Code support:

- **IntelliSense**: Rust-analyzer configuration with correct target settings
- **Debugging**: Probe-rs integration with automatic chip selection
- **Formatting**: Automatic code formatting on save

## 📁 Project Structure

```
src/
├── bin/example.rs                # Demo application
├── board/                        # Board configurations
├── hardware/                     # GPIO, serial, timers
├── service/comm.rs              # Message handling
├── protocol/hdlc.rs             # HDLC framing
└── common/tasks.rs              # Async tasks
```

## 🛠️ Setup & Usage

```bash
# Prerequisites
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add thumbv7em-none-eabihf
cargo install probe-rs-tools --features cli flip-link

# Quick start
git clone <repo-url>
cd embassy-stm32-starter
./setup nucleo                   # Configure board
cargo run --bin example          # Build and flash
```

## 📡 Communication

HDLC protocol with commands: `Ack`, `Nak`, `Ping`, `Raw`. Enable CRC-16: `--features hdlc_fcs`

### Adding New Boards

To support a new STM32 board:

1. **Create board config**: Add `src/board/your_board.rs` following existing patterns
2. **Update templates**: Add board option to template substitution in `setup` script
3. **Test configuration**: Verify memory layout and pin assignments
4. **Document**: Update supported boards table in README

### Code Organization Guidelines

- **Hardware layer**: Board-agnostic utilities in `src/hardware/`
- **Board layer**: MCU-specific code in `src/board/`
- **Service layer**: High-level functionality in `src/service/`
- **Protocol layer**: Communication protocols in `src/protocol/`
- **Application layer**: Binary executables in `src/bin/`

## 📁 Project Structure

```
embassy-stm32-starter/
├── 🎯 setup                           # Board configuration management script
├── 📄 Cargo.toml                     # 🔄 Active project config (managed by setup)
├── 📄 memory.x                       # 🔄 Active memory layout (managed by setup)
├── 📄 board.rs                       # 🔄 Active board config (managed by setup)
├── 📄 rustfmt.toml                   # Code formatting configuration
│
├── 🔧 .cargo/
│   └── config.toml                   # 🔄 Build settings (managed by setup)
│
├── 🖥️ .vscode/                       # VS Code integration
│   ├── launch.json                   # 🔄 Debug config (managed by setup)
│   ├── settings.json                 # Editor settings, rust-analyzer config
│   └── tasks.json                    # Build tasks (cargo check)
│
├── � src/
│   ├── 📄 lib.rs                     # Library root & module exports
│   │
│   ├── 📂 bin/                       # 🎯 Application binaries
│   │   └── example.rs                # Demo app: tasks + communication
│   │
│   ├── 📂 board/                     # Board-specific configurations
│   │   ├── base.rs                   # Common board traits
│   │   ├── nucleo_f446re.rs          # STM32F446RE Nucleo-64 config
│   │   └── nucleo144_f413zh.rs       # STM32F413ZH Nucleo-144 config
│   │
│   ├── 📂 hardware/                  # 🔧 Hardware Abstraction Layer
│   │   ├── flash.rs                  # Flash storage with direct register access
│   │   ├── gpio.rs                   # LED/button control utilities
│   │   ├── hardfault.rs              # Exception handling & auto-reset functionality
│   │   ├── serial.rs                 # UART with DMA + idle detection
│   │   └── timers.rs                 # Timing constants & async delays
│   │
│   ├── 📂 service/                   # 🌐 High-level services
│   │   └── comm.rs                   # HDLC message framing/parsing
│   │
│   ├── 📂 protocol/                  # � Communication protocols
│   │   └── hdlc.rs                   # HDLC frame encode/decode + CRC
│   │
│   └── � common/                    # ♻️ Reusable components
│       └── tasks.rs                  # Embassy async tasks (LED, button, RTC)
│
├── 🧪 tests/                         # Integration testing
│   ├── integration.rs                # Hardware-in-the-loop tests
│   └── flash.rs                      # Flash storage configuration tests
│
└── 📋 Templates/                     # Configuration templates
    ├── Cargo.template.toml           # Cargo config template
    ├── memory.template.x             # Memory layout template
    ├── board.template.rs             # Board config template
    ├── .cargo/config.template.toml   # Build config template
    └── .vscode/launch.template.json  # Debug config template
```

### Key Modules

#### Hardware Layer (`src/hardware/`)

- **`flash.rs`**: Direct STM32F4 register access, bypassing embassy-stm32 v0.4.0 flash bugs, conditional compilation by MCU family
- **`gpio.rs`**: LED control (`LedControl`) and button reading (`ButtonReader`) utilities
- **`hardfault.rs`**: Exception handling with automatic system reset on crashes, debug register logging
- **`serial.rs`**: DMA-based UART with idle interrupt detection, async RX tasks
- **`timers.rs`**: Timing constants and async delay helpers, watchdog feed intervals

#### Service Layer (`src/service/`)

- **`comm.rs`**: High-level message API with HDLC framing, command handling

#### Protocol Layer (`src/protocol/`)

- **`hdlc.rs`**: Low-level HDLC frame encoding/decoding with optional CRC-16

#### Common Tasks (`src/common/`)

- **`tasks.rs`**: Reusable Embassy tasks for LED blinking, button monitoring, RTC clock

#### Board Configurations (`src/board/`)

- **`base.rs`**: Common traits (`BoardConfiguration`, `InterruptHandlers`)
- **`nucleo_f446re.rs`**: STM32F446RE-specific initialization and pin mappings
- **`nucleo144_f413zh.rs`**: STM32F413ZH-specific initialization and pin mappings

## 🛠️ Setup

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

## 🚀 Usage

### Quick Start

```bash
# Clone the project
git clone <repository-url>
cd embassy-stm32-starter

# Configure for your board (defaults to STM32F446RE Nucleo)
./setup nucleo                    # STM32F446RE Nucleo-64
# OR
./setup nucleo144                 # STM32F413ZH Nucleo-144

# Build the project
cargo build --bin example

# Flash and run with RTT logging
cargo run --bin example
```

### Example Application Features

The included `example` binary demonstrates:

- **LED Blinking**: Async task with configurable blink rates
- **Button Monitoring**: Debounced button state detection with logging
- **RTC Clock**: Real-time clock display with timestamp logging
- **Serial Communication**: HDLC message handling with ping/echo responses
- **Watchdog**: Periodic watchdog feeding from main loop

### Available Commands

```bash
# Build commands
cargo build --bin example         # Build example binary
cargo check                      # Quick syntax check
cargo cb                         # Alias for clean
cargo bx example                 # Alias for build --bin example

# Run commands
cargo run --bin example          # Flash and run with RTT logs
cargo start example              # Alias for run --bin example

# Test commands
cargo test --test integration    # Run hardware tests
cargo test --test flash          # Run flash storage tests
cargo tests                     # Alias for test --test integration
```

## 📡 Communication Protocol

### HDLC Message Format

The project implements a custom message protocol over HDLC framing:

```
HDLC Frame: [0x7E] [Escaped Payload] [Escaped CRC-16] [0x7E]

Message Payload (9-byte header + data):
┌─────────┬─────┬───────────┬──────────┬────────┬─────────────┐
│ Command │ ID  │ Fragments │ Fragment │ Length │   Payload   │
│ (u16)   │(u8) │   (u16)   │  (u16)   │ (u16)  │(0-256 bytes)│
└─────────┴─────┴───────────┴──────────┴────────┴─────────────┘
```

### Commands

| Command | Value | Description             |
| ------- | ----- | ----------------------- |
| `Ack`   | 0x01  | Acknowledgment          |
| `Nak`   | 0x02  | Negative acknowledgment |
| `Ping`  | 0x03  | Ping request/response   |
| `Raw`   | 0x04  | Raw data transfer       |

### Flash Storage

Each board uses a dedicated flash sector for persistent storage with **direct register access** to bypass embassy-stm32 v0.4.0 flash driver bugs:

- **STM32F446RE**: Sector (128KB size) - `FLASH_BASE: 0x4002_3C00`
- **STM32F413ZH**: Sector (128KB size) - `FLASH_BASE: 0x4002_3C00`

#### Key Features:

- **Embassy Bug Workaround**: Direct STM32F4 register manipulation bypassing divide-by-zero in embassy flash driver
- **Conditional Compilation**: MCU-specific `FLASH_BASE` addresses via cargo features (`stm32f446`, `stm32f413`)
- **Auto-erase Strategy**: Hardware erase when flash contains data (0xFF writes don't work due to flash physics)

> **⚠️ Flash Pattern**: Demo writes 8-byte patterns throughout entire 128KB storage region. Clean flash (0xFF) → writes data. Dirty flash → erases for next boot. Flash can only change bits 1→0 without erase.

**API:** `flash::erase()` | `flash::write_block()` | `flash::read_block()` | `flash::start()` | `flash::end()`

#### **HardFault Auto-Reset**

- **Crash Detection**: Captures CPU state on crashes (null pointer, invalid memory, stack overflow)

#### **Recovery Flow**

```
Normal: Main Loop → Pet Watchdog (250ms) → Continue → Pet Watchdog...
Hang:   Main Loop Hangs → Watchdog Timeout (1s) → Hardware Reset → Restart
Crash:  HardFault → Log Crash Info → Software Reset → Restart
```

Both systems ensure the MCU automatically recovers from any software or hardware failure without manual intervention.

### Feature Flags

#### **Protocol Features**

```bash
# Enable HDLC CRC-16 verification (enabled by default)
cargo build --features hdlc_fcs
cargo run --features hdlc_fcs --bin example
```

- **`hdlc_fcs` OFF**: Frames include 2-byte trailer, no verification
- **`hdlc_fcs` ON (default)**: PPP/HDLC 16-bit CRC (poly 0x8408) appended and verified

#### **MCU Family Features** (Conditional Compilation)

```bash
# STM32F446RE build (default)
cargo build --features stm32f446

# STM32F413ZH build
cargo build --features stm32f413
```

- **`stm32f446`**: STM32F446RE family - 512KB flash, Nucleo-64 pin mappings
- **`stm32f413`**: STM32F413ZH family - 1536KB flash, Nucleo-144 pin mappings

Features automatically set by `./setup` script based on selected board configuration.

## 🧪 Testing

```bash
cargo test --test integration    # Hardware-in-the-loop tests
cargo test --test flash          # Flash storage configuration tests
```

## 📄 License

Dual licensed under MIT or Apache-2.0 at your option.

## 👤 Author

**Justin L. Hudson** - justinlhudson@gmail.com

---
