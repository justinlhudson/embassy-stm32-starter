// Board configuration for WWZMDiB 2 Pcs STM32F103C8T6 Development Board (Blue Pill)
//
// Board specifications:
// - STM32F103C8T6 MCU (ARM Cortex-M3 @ 72 MHz)
// - 64 KB Flash, 20 KB SRAM
// - LQFP48 package
// - User LED: PC13
// - User Button: (typically PA0)
// - USART1 TX: PA9
// - USART1 RX: PA10

use super::{BoardConfiguration, InterruptHandlers};
use crate::hardware::GpioDefaults;
use crate::hardware::serial;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use embassy_stm32::wdg::IndependentWatchdog;

pub struct BoardConfig;

impl BoardConfiguration for BoardConfig {
  fn board_name() -> &'static str {
    "WWZMDiB STM32F103C8T6 Blue Pill"
  }
}

impl InterruptHandlers for BoardConfig {
  fn setup() {
    // All STM32F103C8T6-specific interrupt handlers are defined below
  }
}

impl BoardConfig {
  /// Busy-wait loop cycles per ms for delays (used by timers.rs)
  pub const fn cycles_per_ms() -> u32 {
    7_200 // 72 MHz
  }
  /// Start address of RAM (for stack usage reporting)
  pub const RAM_START: u32 = 0x20000000;
  /// Watchdog timeout in microseconds
  pub const WATCHDOG_TIMEOUT_US: u32 = 1_000_000;
  /// End address of RAM (for stack usage reporting)
  pub const RAM_END: u32 = 0x20005000; // 20KB RAM ends at 0x20005000
  pub const BOARD_NAME: &'static str = "WWZMDiB STM32F103C8T6 Blue Pill";
  pub const MCU_NAME: &'static str = "STM32F103C8T6";
  pub const FLASH_SIZE_KB: u32 = 64;
  pub const RAM_SIZE_KB: u32 = 20;
  pub const LED_PIN_NAME: &'static str = "PC13";
  pub const LED_DESCRIPTION: &'static str = "User LED (PC13)";
  pub const BUTTON_PIN_NAME: &'static str = "PA0";
  pub const BUTTON_DESCRIPTION: &'static str = "User Button (PA0)";

  /// Initialize LED, button, watchdog, RTC, and serial for this board.
  pub fn init_all_hardware(
    spawner: Spawner,
    p: embassy_stm32::Peripherals,
  ) -> (
    Output<'static>,
    Input<'static>,
    IndependentWatchdog<'static, embassy_stm32::peripherals::IWDG>,
    UartTx<'static, Async>,
  ) {
    // GPIO
    let led = Output::new(p.PC13, GpioDefaults::LED_LEVEL, GpioDefaults::LED_SPEED);
    let button = Input::new(p.PA0, GpioDefaults::BUTTON_PULL);

    // Watchdog
    let mut wdt = IndependentWatchdog::new(p.IWDG, Self::WATCHDOG_TIMEOUT_US);
    wdt.unleash();

    // Serial (USART1 on PA9/PA10)
    let tx = serial::init_serial(spawner, p.USART1, p.PA10, p.PA9, crate::hardware::serial::IrqsUsart1 {}, p.DMA1_CH4, p.DMA1_CH5);

    (led, button, wdt, tx)
  }
}
