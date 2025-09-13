/// GPIO Hardware Abstraction Layer
///
/// This module provides convenient utilities and constants for GPIO operations
/// specific to the STM32F446RE microcontroller setup.
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};

/// LED control utilities
pub struct LedControl;

impl LedControl {
  /// Turn LED on
  pub fn turn_on(led: &mut Output<'_>) {
    led.set_high();
  }

  /// Turn LED off  
  pub fn turn_off(led: &mut Output<'_>) {
    led.set_low();
  }

  /// Toggle LED state
  pub fn toggle(led: &mut Output<'_>) {
    led.toggle();
  }
}

/// Button reading utilities
pub struct ButtonReader;

impl ButtonReader {
  /// Check if button is pressed
  pub fn is_pressed(button: &Input<'_>) -> bool {
    button.is_high()
  }

  /// Check if button is released
  pub fn is_released(button: &Input<'_>) -> bool {
    button.is_low()
  }
}

/// GPIO configuration constants for embedded applications
pub struct GpioDefaults;

impl GpioDefaults {
  /// Standard LED configuration
  pub const LED_LEVEL: Level = Level::Low;
  pub const LED_SPEED: Speed = Speed::Low;

  /// Standard button configuration with pull-down
  pub const BUTTON_PULL: Pull = Pull::Down;
}
