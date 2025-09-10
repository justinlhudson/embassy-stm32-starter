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
use super::{BoardConfiguration, InterruptHandlers};

pub struct BoardConfig;

impl BoardConfiguration for BoardConfig {
	// Board identification
	const BOARD_NAME: &'static str = "STM32 Nucleo-144 F413ZH";
	const MCU_NAME: &'static str = "STM32F413ZH";
    
	// Memory configuration
	const FLASH_SIZE_KB: u32 = 1536;   // 1.5 MB Flash
	const RAM_SIZE_KB: u32 = 320;      // 320 KB SRAM total (256KB + 64KB)
    
	// GPIO Pin assignments
	const LED_PIN_NAME: &'static str = "PB0";   // LD1 - Green LED
	const BUTTON_PIN_NAME: &'static str = "PC13"; // B1 - Blue tactile button
    
	// Pin descriptions
	const LED_DESCRIPTION: &'static str = "Built-in LED LD1 (Green)";
	const BUTTON_DESCRIPTION: &'static str = "Built-in button B1 (Blue)";
    
	/// Initialize all hardware for this board
	fn init_all_hardware(peripherals: embassy_stm32::Peripherals) -> (
		Output<'static>, 
		Input<'static>, 
		embassy_stm32::wdg::IndependentWatchdog<'static, embassy_stm32::peripherals::IWDG>, 
		embassy_stm32::rtc::Rtc
	) {
		use crate::hardware::GpioDefaults;
		use embassy_stm32::rtc::{Rtc, RtcConfig};
		use embassy_stm32::wdg::IndependentWatchdog;
        
		let led = Output::new(peripherals.PB0, GpioDefaults::LED_LEVEL, GpioDefaults::LED_SPEED);
		let button = Input::new(peripherals.PC13, GpioDefaults::BUTTON_PULL);
		let mut wdt = IndependentWatchdog::new(peripherals.IWDG, 1_000_000);
		let rtc = Rtc::new(peripherals.RTC, RtcConfig::default());
        
		wdt.unleash();
        
		(led, button, wdt, rtc)
	}
}

impl InterruptHandlers for BoardConfig {
	fn register_interrupt_handlers() {
		// All STM32F413ZH-specific interrupt handlers are defined below
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
