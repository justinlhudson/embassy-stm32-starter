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

## 🚀 Usage

### Available Commands

```bash
# Configure for your board (defaults to STM32F446RE Nucleo)
./setup nucleo                    # STM32F446RE Nucleo-64
# OR
./setup nucleo144                 # STM32F413ZH Nucleo-144
```

```bash
# Run commands
cargo run --bin example          # Flash and run with RTT logs
# Test commands
cargo test --test <file>         # Run test
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

## 💾 Flash Storage

Each board uses a dedicated flash sector for persistent storage with **direct register access** to bypass embassy-stm32 v0.4.0 flash driver bugs:

#### Key Features:

- **Embassy Bug Workaround**: Direct STM32F4 register manipulation bypassing divide-by-zero in embassy flash driver
- **Conditional Compilation**: MCU-specific `FLASH_BASE` addresses via cargo features (`stm32f446`, `stm32f413`)
- **Auto-erase Strategy**: Hardware erase when flash contains data (0xFF writes don't work due to flash physics)

## 📄 License

Dual licensed under MIT or Apache-2.0 at your option.

## 👤 Author

**Justin L. Hudson** - justinlhudson@gmail.com

---
