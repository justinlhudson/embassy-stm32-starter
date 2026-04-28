# 🚀 Embassy STM32 Starter

> **✨ Magic Setup:** Instantly switch between supported STM32 boards and auto-configure your project with a single command: `./setup <board>`. The setup script updates all configs, linker scripts, and VS Code debug settings for you!

A modern async embedded Rust project template using the **Embassy framework** for STM32 microcontrollers. Features **automatic multi-board configuration**, HDLC communication, comprehensive hardware abstraction, and **automatic crash recovery**.

## ✨ Features

- 🎯 **Multi-Board Support**: STM32F446RE (Nucleo-64) and STM32F413ZH (Nucleo-144)
- 🔄 **One-Command Setup**: Automatic board configuration with `./setup nucleo`
- 📡 **HDLC Communication**: Reliable serial protocol with optional CRC-16 (see [embedded-serial-bridge](https://github.com/justinlhudson/embedded-serial-bridge))
- 💾 **Flash Storage**: Direct register access
- 🛡️ **Auto-Recovery**: Hard fault auto-reset for crash protection
- ⚡ **Async Tasks**: LED, button, RTC, and communication handling
- 🏗️ **Conditional Compilation**: MCU-specific features via cargo flags
- 🔧 **VS Code Ready**: Pre-configured debugging and IntelliSense
- ✅ **Hardware Testing**: Integration tests on real hardware

## 🏗️ Architecture & Configuration

### 🧩 Heapless Design

This project uses the [`heapless`](https://docs.rs/heapless) crate for all dynamic data structures, such as `heapless::Vec`.

### Supported Boards

| Board          | MCU         | Flash  | RAM   | Serial | LED | Button | Flash Storage  |
| -------------- | ----------- | ------ | ----- | ------ | --- | ------ | -------------- |
| **Nucleo-64**  | STM32F446RE | 512KB  | 128KB | USART2 | PA5 | PC13   | Sector (128KB) |
| **Nucleo-144** | STM32F413ZH | 1536KB | 320KB | USART3 | PB0 | PC13   | Sector (128KB) |

## 📁 Project Structure

```
embassy-stm32-starter/
├── 🎯 setup                           # Board configuration & dependency install script
├── ⚡ flash                           # Build, flash, and stream RTT logs
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

## � Applications

This starter includes example applications demonstrating different use cases:

### 🎯 `example` - Full Feature Demo

Located in `src/bin/example.rs`, this comprehensive demo showcases all framework capabilities:

- All async tasks (LED blinking, button handling, RTC)
- Flash storage operations
- HDLC communication protocol with message handling (Ping, Raw commands)
- Integration with all hardware modules
- Auto-recovery on HDLC FCS errors

### 🔌 `relay` - GPIO Control & Communication

Located in `src/bin/relay.rs`, a focused application derived from `example` for remote GPIO control:

- **Button Control**: Toggle D8 output (PA9 on Nucleo-\*) using the onboard button
- **Serial Control**: Control D8 via HDLC Raw commands (`0xD8 0x01` = HIGH, `0xD8 0x00` = LOW)

Use `cargo run --bin relay` to flash and run the relay application.

## �🚀 Usage

### Commands

### `setup` — one-time board configuration

Configures the project for a target board, installs all required dependencies
(`arm-none-eabi-ld`, `flip-link`, `probe-rs`, Rust cross target), and regenerates
derived config files (`Cargo.toml`, `memory.x`, `.cargo/config.toml`, `.vscode/launch.json`).

```bash
./setup nucleo                    # STM32F446RE Nucleo-64  (default)
./setup nucleo144                 # STM32F413ZH Nucleo-144
```

Re-run whenever you switch boards or on a fresh clone.

### `flash` — build, flash, and stream logs

Detects the connected board, calls `setup` if needed, builds the selected binary,
flashes it via `probe-rs`, and streams RTT logs until Ctrl+C.

```bash
./flash                           # flash default binary (example)
./flash relay                     # flash a specific binary
```

### Other commands

```bash
cargo run --bin example          # alternative: flash and run via cargo
cargo test --test <file>         # run integration tests
```

## 📡 Communication Protocol

### HDLC Message Format

The project implements a custom message protocol over HDLC framing (see [embedded-serial-bridge](https://github.com/justinlhudson/embedded-serial-bridge) for PC client implementation):

```
HDLC Frame: [0x7E] [Escaped Payload] [Escaped CRC-16] [0x7E]

Message Payload (9-byte header + data):
┌─────────┬─────┬───────────┬──────────┬────────┬─────────────┐
│ Command │ ID  │ Fragments │ Fragment │ Length │   Payload   │
│ (u16)   │(u8) │   (u16)   │  (u16)   │ (u16)  │(0-256 bytes)│
└─────────┴─────┴───────────┴──────────┴────────┴─────────────┘
```

### Commands (initial)

| Command | Value | Description             |
| ------- | ----- | ----------------------- |
| `Ack`     | 0x01  | Acknowledgment                    |
| `Nak`     | 0x02  | Negative acknowledgment           |
| `Ping`    | 0x03  | Ping request/response             |
| `Raw`     | 0x04  | Raw data transfer                 |
| `Version` | 0x05  | Query firmware version (semver)   |

## 💾 Flash Storage

Each board uses a dedicated flash sector for persistent storage with **direct register access**.

#### Key Features:

- **Conditional Compilation**: MCU-specific `FLASH_BASE` addresses via cargo features (`stm32f446`, `stm32f413`)
- **Auto-erase Strategy**: Hardware erase when flash contains data (0xFF writes don't work due to flash physics)

## 📄 License

Dual licensed under MIT or Apache-2.0 at your option.

## 👤 Author

**Justin L. Hudson** - justinlhudson@gmail.com

---
