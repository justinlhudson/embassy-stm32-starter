//! GPIO defaults shared across boards.
//!
//! The previous `LedControl`/`ButtonReader` wrappers added no value over
//! calling [`embassy_stm32::gpio::Output`]/[`Input`] methods directly, so
//! they were removed. Use `led.set_high()`, `button.is_high()`, etc.

use embassy_stm32::gpio::{Level, Pull, Speed};

pub struct GpioDefaults;

impl GpioDefaults {
  /// Default LED state at boot.
  pub const LED_LEVEL: Level = Level::Low;
  /// Default LED slew rate.
  pub const LED_SPEED: Speed = Speed::Low;
  /// Default button pull (Nucleo B1 wiring varies; pull-down works for both
  /// supported boards because the button drives the line high when pressed).
  pub const BUTTON_PULL: Pull = Pull::Down;
}
