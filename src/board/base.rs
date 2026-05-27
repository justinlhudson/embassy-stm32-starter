//! Board abstraction: every supported board implements [`Board`] with a
//! complete set of constants plus an `init()` that returns [`BoardHardware`].
//!
//! Adding a new board: implement [`Board`] for a unit struct, then wire it
//! up in `src/board/mod.rs` (or `src/lib.rs`'s `board` module).

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::mode::Async;
use embassy_stm32::peripherals::IWDG;
use embassy_stm32::rtc::RtcTimeProvider;
use embassy_stm32::usart::UartTx;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Peripherals;

/// Hardware handles returned by [`Board::init`].
pub struct BoardHardware {
  pub led: Output<'static>,
  pub button: Input<'static>,
  pub wdt: IndependentWatchdog<'static, IWDG>,
  /// RTC time-provider handle. `Rtc` itself is consumed by the global RTC
  /// container inside embassy; `RtcTimeProvider` is the cheap, shareable
  /// reader that exposes [`RtcTimeProvider::now`].
  pub rtc: RtcTimeProvider,
  pub tx: UartTx<'static, Async>,
}

/// Full board interface. The compiler enforces that every board provides
/// every constant and the `init` function — no more silent drift.
pub trait Board {
  // --- identity ---
  const BOARD_NAME: &'static str;
  const MCU_NAME: &'static str;

  // --- sizing (for stack-usage reporting, etc.) ---
  const FLASH_SIZE_KB: u32;
  const RAM_SIZE_KB: u32;
  const RAM_START: u32;
  const RAM_END: u32;

  // --- persistent storage region (typically the last 128KB flash sector) ---
  /// Absolute address of the storage region start.
  const FLASH_STORAGE_START: u32;
  /// Absolute address of the storage region end (exclusive).
  const FLASH_STORAGE_END: u32;
  /// Convenience: byte size of the storage region.
  const FLASH_STORAGE_SIZE: usize = (Self::FLASH_STORAGE_END - Self::FLASH_STORAGE_START) as usize;

  // --- watchdog ---
  const WATCHDOG_TIMEOUT_US: u32;

  // --- documentation strings (for boot-time logging) ---
  const LED_PIN_NAME: &'static str;
  const LED_DESCRIPTION: &'static str;
  const BUTTON_PIN_NAME: &'static str;
  const BUTTON_DESCRIPTION: &'static str;

  /// Initialize LED, button, watchdog, RTC, and serial. The serial
  /// implementation spawns its own RX/HDLC tasks via the supplied spawner.
  ///
  /// IMPORTANT: the watchdog is **not** unleashed here. Callers that perform
  /// flash erase (up to ~4s on STM32F4) must do so before calling
  /// `wdt.unleash()`; the IWDG cannot be stopped once started.
  fn init(spawner: Spawner, p: Peripherals) -> BoardHardware;
}
