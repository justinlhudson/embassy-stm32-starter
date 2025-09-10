// Board configuration for STM32 Nucleo-64 Development Board with STM32F446RE
//
// Board specifications:
// - STM32F446RE MCU (ARM Cortex-M4F @ 180 MHz)
// - 512 KB Flash, 128 KB SRAM
// - LQFP64 package
// - Built-in ST-LINK/V2-1 debugger
// - Arduino Uno R3 and ST morpho connector compatibility
// - User LED and button
//
// Pin assignments for Nucleo-F446RE:
// - User LED (LD2): PA5 (Green LED)
// - User Button (B1): PC13 (Blue tactile button)
// - USART2 TX: PA2
// - USART2 RX: PA3

// use embassy_stm32::gpio::{Input, Output};
// use embassy_stm32::peripherals;
use super::{BoardConfiguration, InterruptHandlers};

pub struct BoardConfig;

impl BoardConfig {
    // Board constants (for compatibility with existing applications)
    pub const BOARD_NAME: &'static str = "STM32 Nucleo-64 F446RE";
    pub const MCU_NAME: &'static str = "STM32F446RE";
    pub const FLASH_SIZE_KB: u32 = 512;
    pub const RAM_SIZE_KB: u32 = 128;
    pub const LED_PIN_NAME: &'static str = "PA5";
    pub const LED_DESCRIPTION: &'static str = "Green User LED (LD2)";
    pub const BUTTON_PIN_NAME: &'static str = "PC13";
    pub const BUTTON_DESCRIPTION: &'static str = "Blue User Button (B1)";
    
    // TODO: Implement init_all_hardware method
    // pub fn init_all_hardware(peripherals: embassy_stm32::Peripherals) -> (...) {
    //     // Return (led, button, watchdog, rtc, serial) 
    // }
}

impl BoardConfiguration for BoardConfig {
	fn board_name() -> &'static str {
		"STM32 Nucleo-64 F446RE"
	}
}

impl InterruptHandlers for BoardConfig {
	fn setup() {
		// All STM32F446RE-specific interrupt handlers are defined below
	}
}

// STM32F446RE interrupt handlers - required for linking
#[unsafe(no_mangle)]
extern "C" fn DefaultHandler() {
    // Default handler - just hang
    loop {}
}

// Provide default implementations for the missing interrupt handlers
#[unsafe(no_mangle)]
extern "C" fn PVD() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn OTG_HS_EP1_OUT() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn OTG_HS_EP1_IN() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn OTG_HS_WKUP() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn OTG_HS() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn SAI1() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn SAI2() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn QUADSPI() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn CEC() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn SPDIF_RX() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn FMPI2C1_EV() {
    DefaultHandler();
}

#[unsafe(no_mangle)]
extern "C" fn FMPI2C1_ER() {
    DefaultHandler();
}
