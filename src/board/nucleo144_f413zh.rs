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

use embassy_stm32::gpio::{Input, Output};
use embassy_executor::Spawner;
use embassy_stm32::mode::Async;
use embassy_stm32::usart::UartTx;
use embassy_stm32::rtc::{Rtc, RtcConfig};
use embassy_stm32::wdg::IndependentWatchdog;
use crate::hardware::serial;
use crate::hardware::GpioDefaults;
use super::{BoardConfiguration, InterruptHandlers};

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
	// Board constants (mirroring F446RE style)
	pub const MCU_NAME: &'static str = "STM32F413ZH";
	pub const FLASH_SIZE_KB: u32 = 1536;   // 1.5 MB Flash
	pub const RAM_SIZE_KB: u32 = 320;      // 320 KB SRAM total (256KB + 64KB)
	pub const LED_PIN_NAME: &'static str = "PB0";   // LD1 - Green LED
	pub const LED_DESCRIPTION: &'static str = "Built-in LED LD1 (Green)";
	pub const BUTTON_PIN_NAME: &'static str = "PC13"; // B1 - Blue tactile button
	pub const BUTTON_DESCRIPTION: &'static str = "Built-in button B1 (Blue)";

	/// Initialize USART3 serial for this board (PD9=TX, PD8=RX), spawn RX/HDLC tasks, and return TX half
	pub fn init_serial(spawner: Spawner, p: embassy_stm32::Peripherals) -> UartTx<'static, Async> {
		// On STM32F413ZH Nucleo-144, default VCP often maps to USART3 (PD8=RX, PD9=TX)
		// DMA mapping (common on F4): TX = DMA1_CH3, RX = DMA1_CH1 (adjust if needed per actual board schematic)
		serial::init_serial(
			spawner,
			p.USART3,
			p.PD8,  // RX
			p.PD9,  // TX
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
		let mut wdt = IndependentWatchdog::new(p.IWDG, 1_000_000);
		let rtc = Rtc::new(p.RTC, RtcConfig::default());
		wdt.unleash();

		// Serial (USART3 on PD9/PD8)
	let comm = serial::init_serial(
			spawner,
			p.USART3,
			p.PD8,         // RX
			p.PD9,         // TX
			serial::Serial3Irqs,
			p.DMA1_CH3,    // TX DMA for USART3
			p.DMA1_CH1,    // RX DMA for USART3
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
