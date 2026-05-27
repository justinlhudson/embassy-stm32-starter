//! STM32 Nucleo-144 with STM32F413ZH.
//!
//! - LED:    PB0 (LD1, green)
//! - Button: PC13 (B1, blue)
//! - Serial: USART3 on PD8/PD9 (ST-LINK VCP), DMA1_CH3 (TX), DMA1_CH1 (RX)

use super::base::{Board, BoardHardware};
use crate::hardware::serial;
use crate::hardware::GpioDefaults;
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_stm32::Peripherals;
use embassy_stm32::{bind_interrupts, dma, usart};

bind_interrupts!(struct Usart3Irqs {
  USART3       => usart::InterruptHandler<embassy_stm32::peripherals::USART3>;
  DMA1_STREAM3 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH3>;
  DMA1_STREAM1 => dma::InterruptHandler<embassy_stm32::peripherals::DMA1_CH1>;
});

pub struct BoardConfig;

impl Board for BoardConfig {
  const BOARD_NAME: &'static str = "STM32 Nucleo-144 F413ZH";
  const MCU_NAME: &'static str = "STM32F413ZH";
  const FLASH_SIZE_KB: u32 = 1536;
  const RAM_SIZE_KB: u32 = 320;
  const RAM_START: u32 = 0x2000_0000;
  const RAM_END: u32 = 0x2005_0000; // 320KB
  /// Last 128KB sector (sector 15) of the 1536KB flash.
  const FLASH_STORAGE_START: u32 = 0x0816_0000;
  const FLASH_STORAGE_END: u32 = 0x0818_0000;
  const WATCHDOG_TIMEOUT_US: u32 = 1_000_000;
  const LED_PIN_NAME: &'static str = "PB0";
  const LED_DESCRIPTION: &'static str = "Built-in LED LD1 (Green)";
  const BUTTON_PIN_NAME: &'static str = "PC13";
  const BUTTON_DESCRIPTION: &'static str = "Built-in button B1 (Blue)";

  fn init(spawner: Spawner, p: Peripherals) -> BoardHardware {
    let led = Output::new(p.PB0, GpioDefaults::LED_LEVEL, GpioDefaults::LED_SPEED);
    let button = Input::new(p.PC13, GpioDefaults::BUTTON_PULL);
    let wdt = IndependentWatchdog::new(p.IWDG, Self::WATCHDOG_TIMEOUT_US);
    let (_rtc, rtc) = Rtc::new(p.RTC, RtcConfig::default());

    // Hand the FLASH peripheral to the global flash driver before the app
    // tries to read/write/erase. Cheap; just stores it in a Mutex<RefCell>.
    crate::hardware::flash::init(p.FLASH);

    let tx = serial::init_serial(spawner, p.USART3, p.PD9, p.PD8, p.DMA1_CH3, p.DMA1_CH1, Usart3Irqs);

    BoardHardware { led, button, wdt, rtc, tx }
  }
}

// --- STM32F413ZH-specific interrupt stubs (avoid linker errors) ---
macro_rules! stub_irq { ($($name:ident),* $(,)?) => { $(
  #[unsafe(no_mangle)] extern "C" fn $name() {}
)* } }

stub_irq!(
  WWDG,
  I2C1_EV,
  I2C1_ER,
  I2C2_EV,
  I2C2_ER,
  RTC_ALARM,
  OTG_FS_WKUP,
  SPI3,
  TIM6_DAC,
  LPTIM1,
  DFSDM2_FLT0,
  DFSDM2_FLT1,
  DFSDM2_FLT2,
  DFSDM2_FLT3,
  QUADSPI,
  FMPI2C1_EV,
  FMPI2C1_ER,
);
