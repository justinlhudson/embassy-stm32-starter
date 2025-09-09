// Base board configuration module - defines the common interface for all board implementations
//
// This module provides the base traits and validation that all board configurations must implement
// to ensure consistency and completeness across different hardware platforms.
//
// All board-specific implementations should:
// 1. Implement the BoardConfiguration trait
// 2. Implement the InterruptHandlers trait  
// 3. Use the validation macro to ensure compile-time checks
// 4. Follow the established naming and structure conventions

use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::rtc::Rtc;
use embassy_stm32::wdg::IndependentWatchdog;

/// Base trait that all board configurations must implement
/// 
/// This trait enforces a consistent interface across all supported boards,
/// ensuring that each board provides all required constants and methods.
pub trait BoardConfiguration {
    // Required board identification constants
    const BOARD_NAME: &'static str;
    const MCU_NAME: &'static str;
    
    // Required memory configuration constants
    const FLASH_SIZE_KB: u32;
    const RAM_SIZE_KB: u32;
    
    // Required GPIO pin assignments
    const LED_PIN_NAME: &'static str;
    const BUTTON_PIN_NAME: &'static str;
    
    // Required pin descriptions for documentation
    const LED_DESCRIPTION: &'static str;
    const BUTTON_DESCRIPTION: &'static str;
    
    /// Initialize all required hardware peripherals for this board
    /// 
    /// Returns a tuple containing:
    /// - LED output pin
    /// - Button input pin  
    /// - Independent watchdog timer
    /// - Real-time clock
    /// - Async serial UART
    fn init_all_hardware(
        peripherals: embassy_stm32::Peripherals
    ) -> (
        Output<'static>,
        Input<'static>,
        IndependentWatchdog<'static, embassy_stm32::peripherals::IWDG>,
        Rtc,
        embassy_stm32::usart::Uart<'static, embassy_stm32::mode::Async>
    );
    
    /// Get board information as a formatted string
    fn board_info() -> &'static str {
        Self::BOARD_NAME
    }
    
    /// Get memory information summary
    fn memory_info() -> (u32, u32) {
        (Self::FLASH_SIZE_KB, Self::RAM_SIZE_KB)
    }
    
    /// Validate board configuration at compile time
    /// 
    /// This method performs compile-time checks to ensure the board
    /// configuration is valid and complete.
    fn validate_config() -> bool {
        // Ensure all required constants are non-empty
        !Self::BOARD_NAME.is_empty() 
            && !Self::MCU_NAME.is_empty()
            && !Self::LED_PIN_NAME.is_empty()
            && !Self::BUTTON_PIN_NAME.is_empty()
            && Self::FLASH_SIZE_KB > 0
            && Self::RAM_SIZE_KB > 0
    }
}

/// Macro to implement basic validation for board configs
/// 
/// This macro can be used by board implementations to automatically
/// generate compile-time validation checks using const assertions.
#[macro_export]
macro_rules! validate_board_config {
    ($board_type:ty) => {
        // Use const assertions to validate at compile time
        const _: () = assert!(!<$board_type>::BOARD_NAME.is_empty());
        const _: () = assert!(!<$board_type>::MCU_NAME.is_empty());
        const _: () = assert!(!<$board_type>::LED_PIN_NAME.is_empty());
        const _: () = assert!(!<$board_type>::BUTTON_PIN_NAME.is_empty());
        const _: () = assert!(<$board_type>::FLASH_SIZE_KB > 0);
        const _: () = assert!(<$board_type>::RAM_SIZE_KB > 0);
    };
}

/// Common interrupt handler trait
/// 
/// Boards should implement this to provide their MCU-specific interrupt handlers.
/// This helps ensure all required interrupts are handled consistently.
pub trait InterruptHandlers {
    /// Register all MCU-specific interrupt handlers
    /// 
    /// This method should contain all the `extern "C"` interrupt handler
    /// functions required for the specific MCU.
    fn register_interrupt_handlers();
}
