//! STM32 Nucleo-64 with STM32F446RE.
//!
//! - LED:    PA5 (LD2, green)
//! - Button: PC13 (B1, blue)
//! - Serial: USART2 on PA2/PA3 (ST-LINK VCP), DMA1_CH6 (TX), DMA1_CH5 (RX)

use super::base::{Board, BoardHardware};
use crate::hardware::serial;
use crate::hardware::GpioDefaults;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Peripherals;
use embassy_stm32::{bind_interrupts, dma, usart};

bind_interrupts!(struct Usart2Irqs {
  USART2       => usart::InterruptHandler<embassy_stm32::peripherals::USART2>;
  DMA1_STREAM5 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH5>;
  DMA1_STREAM6 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH6>;
});

pub struct BoardConfig;

impl Board for BoardConfig {
  const BOARD_NAME: &'static str = "STM32 Nucleo-64 F446RE";
  const MCU_NAME: &'static str = "STM32F446RE";
  const FLASH_SIZE_KB: u32 = 512;
  const RAM_SIZE_KB: u32 = 128;
  const RAM_START: u32 = 0x2000_0000;
  const RAM_END: u32 = 0x2002_0000; // 128KB
  /// Sector 6 (128KB) of the 512KB flash.
  const FLASH_STORAGE_START: u32 = 0x0804_0000;
  const FLASH_STORAGE_END: u32 = 0x0806_0000;
  const WATCHDOG_TIMEOUT_US: u32 = 1_000_000;
  const LED_PIN_NAME: &'static str = "PA5";
  const LED_DESCRIPTION: &'static str = "Green User LED (LD2)";
  const BUTTON_PIN_NAME: &'static str = "PC13";
  const BUTTON_DESCRIPTION: &'static str = "Blue User Button (B1)";

  fn init(spawner: Spawner, p: Peripherals) -> BoardHardware {
    let led = Output::new(p.PA5, GpioDefaults::LED_LEVEL, GpioDefaults::LED_SPEED);
    let button = Input::new(p.PC13, GpioDefaults::BUTTON_PULL);
    let wdt = IndependentWatchdog::new(p.IWDG, Self::WATCHDOG_TIMEOUT_US);
    let (_rtc, rtc) = Rtc::new(p.RTC, RtcConfig::default());

    crate::hardware::flash::init(p.FLASH);

    let tx = serial::init_serial(spawner, p.USART2, p.PA3, p.PA2, p.DMA1_CH6, p.DMA1_CH5, Usart2Irqs);

    BoardHardware { led, button, wdt, rtc, tx }
  }
}

// --- STM32F446RE-specific interrupt stubs (avoid linker errors) ---
macro_rules! stub_irq { ($($name:ident),* $(,)?) => { $(
  #[unsafe(no_mangle)] extern "C" fn $name() {}
)* } }

stub_irq!(
  PVD,
  OTG_HS_EP1_OUT,
  OTG_HS_EP1_IN,
  OTG_HS_WKUP,
  OTG_HS,
  SAI1,
  SAI2,
  QUADSPI,
  CEC,
  SPDIF_RX,
  FMPI2C1_EV,
  FMPI2C1_ER,
);
