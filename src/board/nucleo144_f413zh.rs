// Board configuration for STM32 Nucleo-144 Development Board with STM32F413ZH
// Board configuration for STM32 Nucleo-144 Development Board with STM32F413ZH
//
// Board specifications:
// - STM32F413ZH MCU (ARM Cortex-M4F @ 100 MHz)
// - 1536 KB Flash, 320 KB SRAM
// - LQFP144 package with extensive GPIO
// - Built-in ST-LINK/V2-1 debugger
// - Arduino Uno R3 and ST morpho connector compatibility
// - Multiple user LEDs and buttons
//
// Pin assignments for Nucleo-144 F413ZH:
// - User LED1 (LD1): PB0  (Green LED)
// - User LED2 (LD2): PB7  (Blue LED)
// - User LED3 (LD3): PB14 (Red LED)
// - User Button (B1): PC13 (Blue tactile button)
//
// Note: This board has 3 user LEDs, we'll use LD1 (Green) as the primary LED

use super::{BoardConfiguration, InterruptHandlers};
use crate::hardware::GpioDefaults;
use crate::hardware::serial;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::mode::Async;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::usart::UartTx;
use embassy_stm32::wdg::IndependentWatchdog;

use embassy_stm32::Config as EmbassyConfig;
// Advanced RCC configuration disabled for compatibility

pub struct BoardConfig;

// Implement the minimal trait per base.rs
impl BoardConfiguration for BoardConfig {
  fn board_name() -> &'static str {
    "STM32 Nucleo-144 F413ZH"
  }
}

impl InterruptHandlers for BoardConfig {
  fn setup() {
    // All STM32F413ZH-specific interrupt handlers are defined below
  }
}

impl BoardConfig {
  /// Returns the default Embassy config (16 MHz HSI)
  /// Note: Advanced clock configuration disabled due to embassy-stm32 API changes
  pub fn embassy_config() -> EmbassyConfig {
    EmbassyConfig::default()
  }
  /// Busy-wait loop cycles per ms for delays (used by timers.rs)
  pub const fn cycles_per_ms() -> u32 {
    0 // Not used (async timer available)
  }
  /// Start address of RAM (for stack usage reporting)
  pub const RAM_START: u32 = 0x20000000;
  /// Watchdog timeout in microseconds
  pub const WATCHDOG_TIMEOUT_US: u32 = 1_000_000;
  /// End address of RAM (for stack usage reporting)
  pub const RAM_END: u32 = 0x20050000; // 320KB RAM ends at 0x20050000

  /// Flash storage region: Use last 128KB sector of STM32F413ZH (1536KB flash)
  /// STM32F413ZH flash: 1536KB total (0x08000000 to 0x08180000)
  /// Using last 128KB for storage: 1408KB to 1536KB from flash base
  pub const FLASH_STORAGE_START: u32 = 0x08160000; // Start of last 128KB (1408KB from base)
  pub const FLASH_STORAGE_END: u32 = 0x08180000; // End of flash (1536KB from base)
  pub const FLASH_STORAGE_SIZE: usize = 128 * 1024; // 128KB storage region
  // Board constants (mirroring F446RE style)
  pub const BOARD_NAME: &'static str = "STM32 Nucleo-144 F413ZH";
  pub const MCU_NAME: &'static str = "STM32F413ZH";
  pub const FLASH_SIZE_KB: u32 = 1536; // 1.5 MB Flash
  pub const RAM_SIZE_KB: u32 = 320; // 320 KB SRAM total (256KB + 64KB CCM)
  pub const LED_PIN_NAME: &'static str = "PB0"; // LD1 - Green LED
  pub const LED_DESCRIPTION: &'static str = "Built-in LED LD1 (Green)";
  pub const BUTTON_PIN_NAME: &'static str = "PC13"; // B1 - Blue tactile button
  pub const BUTTON_DESCRIPTION: &'static str = "Built-in button B1 (Blue)";

  /// Initialize USART3 serial for this board (PD8=TX, PD9=RX) - ST-LINK VCP, spawn RX/HDLC tasks, and return TX half
  pub fn init_serial(spawner: Spawner, p: embassy_stm32::Peripherals) -> UartTx<'static, Async> {
    // On STM32F413ZH Nucleo-144, using USART3 (PD9=RX, PD8=TX) for ST-LINK VCP
    // DMA mapping for USART3: TX = DMA1_CH3, RX = DMA1_CH1
    serial::init_serial(
      spawner,
      p.USART3,
      p.PD9, // RX
      p.PD8, // TX
      serial::Serial3Irqs,
      p.DMA1_CH3, // TX DMA for USART3
      p.DMA1_CH1, // RX DMA for USART3
    )
  }

  /// Initialize LED, button, watchdog, RTC, and serial for this board.
  pub fn init_all_hardware(
    spawner: Spawner,
    p: embassy_stm32::Peripherals,
  ) -> (
    Output<'static>,
    Input<'static>,
    IndependentWatchdog<'static, embassy_stm32::peripherals::IWDG>,
    Rtc,
    UartTx<'static, Async>,
  ) {
    // GPIO
    let led = Output::new(p.PB0, GpioDefaults::LED_LEVEL, GpioDefaults::LED_SPEED);
    let button = Input::new(p.PC13, GpioDefaults::BUTTON_PULL);

    // Watchdog and RTC
    let mut wdt = IndependentWatchdog::new(p.IWDG, Self::WATCHDOG_TIMEOUT_US);
    let rtc = Rtc::new(p.RTC, RtcConfig::default());
    wdt.unleash();

    // Serial (USART3 on PD8/PD9 - ST-LINK VCP)
    let comm = serial::init_serial(
      spawner,
      p.USART3,
      p.PD9, // RX
      p.PD8, // TX
      serial::Serial3Irqs,
      p.DMA1_CH3, // TX DMA for USART3
      p.DMA1_CH1, // RX DMA for USART3
    );

    (led, button, wdt, rtc, comm)
  }
}

// Compile-time validation
crate::validate_board_config!(BoardConfig);

// STM32F413ZH-specific interrupt handlers
#[unsafe(no_mangle)]
extern "C" fn WWDG() {}

#[unsafe(no_mangle)]
extern "C" fn I2C1_EV() {}

#[unsafe(no_mangle)]
extern "C" fn I2C1_ER() {}

#[unsafe(no_mangle)]
extern "C" fn I2C2_EV() {}

#[unsafe(no_mangle)]
extern "C" fn I2C2_ER() {}

#[unsafe(no_mangle)]
extern "C" fn RTC_ALARM() {}

#[unsafe(no_mangle)]
extern "C" fn OTG_FS_WKUP() {}

#[unsafe(no_mangle)]
extern "C" fn SPI3() {}

#[unsafe(no_mangle)]
extern "C" fn TIM6_DAC() {}

#[unsafe(no_mangle)]
extern "C" fn LPTIM1() {}

#[unsafe(no_mangle)]
extern "C" fn DFSDM2_FLT0() {}

#[unsafe(no_mangle)]
extern "C" fn DFSDM2_FLT1() {}

#[unsafe(no_mangle)]
extern "C" fn DFSDM2_FLT2() {}

#[unsafe(no_mangle)]
extern "C" fn DFSDM2_FLT3() {}

// Additional shared interrupts that F413ZH also needs
#[unsafe(no_mangle)]
extern "C" fn QUADSPI() {}

#[unsafe(no_mangle)]
extern "C" fn FMPI2C1_EV() {}

#[unsafe(no_mangle)]
extern "C" fn FMPI2C1_ER() {}
