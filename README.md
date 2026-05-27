# Embassy STM32 Starter

Async embedded Rust template for STM32 Nucleo boards built on the
[Embassy](https://embassy.dev) framework. Multi-board support is driven by
Cargo features and a `build.rs` — no setup script required.

## Features

- Multi-board: STM32F413ZH (Nucleo-144) and STM32F446RE (Nucleo-64)
- Compile-time board selection via Cargo features
- HDLC framing with optional CRC-16 FCS (see
  [embedded-serial-bridge](https://github.com/justinlhudson/embedded-serial-bridge))
- Flash storage via the official `embassy-stm32` driver
- Hard-fault auto-reset
- Async tasks for LED, button (edge-debounced), RTC, and comms
- Two example binaries: `example` (full feature tour) and `relay` (HDLC-controlled GPIO)
- HIL tests, including an opt-in destructive flash round-trip

## Supported Boards

| Board      | MCU         | Flash  | RAM   | Serial | LED | Button | Storage sector |
| ---------- | ----------- | ------ | ----- | ------ | --- | ------ | -------------- |
| Nucleo-144 | STM32F413ZH | 1536KB | 320KB | USART3 | PB0 | PC13   | 128KB (S15)    |
| Nucleo-64  | STM32F446RE |  512KB | 128KB | USART2 | PA5 | PC13   | 128KB (S6)     |

## Project Layout

```
embassy-stm32-starter/
├── build.rs                  # picks the right memory.x from /memory/ based on MCU feature
├── memory/                   # canonical linker scripts per MCU
│   ├── stm32f413zh.x
│   └── stm32f446re.x
├── flash.sh                  # auto-detects board via DBGMCU IDCODE, builds + flashes
├── Cargo.toml                # feature-driven; pick one of `stm32f413` / `stm32f446`
├── src/
│   ├── lib.rs                # crate root + module tree + `prelude`
│   ├── bin/
│   │   ├── example.rs        # full demo: tasks, flash, comms
│   │   └── relay.rs          # HDLC-controlled GPIO (PA9 / Arduino D8)
│   ├── board/
│   │   ├── base.rs           # `Board` trait + `BoardHardware` struct
│   │   ├── nucleo144_f413zh.rs
│   │   └── nucleo_f446re.rs
│   ├── hardware/
│   │   ├── flash.rs          # `Flash::new_blocking()` wrapper + region helpers
│   │   ├── gpio.rs           # `GpioDefaults` constants
│   │   ├── hardfault.rs      # auto-reset on fault
│   │   ├── serial.rs         # UART + DMA + idle-line RX task
│   │   └── timers.rs         # `Timing` constants + helpers
│   ├── protocol/
│   │   └── hdlc.rs           # framer + deframer + CRC-16
│   ├── service/
│   │   └── comm.rs           # message struct, parser, `dispatch()` helper
│   └── common/
│       └── tasks.rs          # reusable tasks: `button_monitor`, `rtc_clock`, `led_blink`
└── tests/
    ├── integration.rs
    └── flash.rs              # destructive ops behind `flash-destructive` feature
```

## Build & Flash

### Auto-detect board

```bash
./flash.sh              # builds `example`, detects the connected MCU, flashes
./flash.sh relay        # flashes the `relay` binary
```

`flash.sh` reads the DBGMCU IDCODE via `probe-rs` and passes the right
`--features` set to `cargo`.

### Manual

```bash
cargo build --release --no-default-features --features stm32f413,hdlc_fcs
cargo build --release --no-default-features --features stm32f446,hdlc_fcs
```

## Features

| Feature             | Purpose                                                |
| ------------------- | ------------------------------------------------------ |
| `stm32f413`         | Target STM32F413ZH (Nucleo-144)                        |
| `stm32f446`         | Target STM32F446RE (Nucleo-64)                         |
| `hdlc_fcs`          | Append/verify CRC-16 FCS on every HDLC frame           |
| `flash-destructive` | Allow the `flash` HIL test to erase + write the sector |

Exactly one MCU feature must be enabled; the build will `compile_error!`
otherwise.

## Communication Protocol

```
HDLC frame:  [0x7E] [escaped payload] [escaped CRC-16] [0x7E]

Message payload (9-byte header + 0..=256 B body, little-endian):
┌─────────┬─────┬───────────┬──────────┬────────┬─────────────┐
│ Command │ ID  │ Fragments │ Fragment │ Length │   Payload   │
│  (u16)  │(u8) │   (u16)   │  (u16)   │ (u16)  │             │
└─────────┴─────┴───────────┴──────────┴────────┴─────────────┘
```

| Command   | Value | Description                                        |
| --------- | ----- | -------------------------------------------------- |
| `Ack`     | 0x01  | Acknowledgment                                     |
| `Nak`     | 0x02  | Negative acknowledgment                            |
| `Ping`    | 0x03  | Auto-replied by `comm::dispatch` (echoes the msg)  |
| `Raw`     | 0x04  | Application-defined payload                        |
| `Version` | 0x05  | Auto-replied by `comm::dispatch` (`CARGO_PKG_VERSION`) |

The library never resets the MCU on FCS errors — that's an app-level
policy. `relay.rs` shows one such policy; `example.rs` just logs.

## Flash Storage

`hardware::flash` wraps `embassy_stm32::flash::Flash::new_blocking()` with
a per-board storage region (see `Board::FLASH_STORAGE_{START,END}`).

> A 128 KB sector erase on STM32F4 can take ~4 seconds. Run any erase
> **before** `wdt.unleash()` — the IWDG cannot be disarmed once started,
> and a sector erase mid-window will trigger a reset.

## License

Dual licensed under MIT or Apache-2.0 at your option.

## Author

Justin L. Hudson — justinlhudson@gmail.com
