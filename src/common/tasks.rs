//! Reusable Embassy tasks that any binary can spawn.
//!
//! These intentionally stay tiny — copy them into your own binary if you
//! need different behaviour rather than growing this module.

use crate::hardware::Timing;
use crate::*;
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::rtc::RtcTimeProvider;

/// LED blinking task — toggles `led` every `delay_ms` milliseconds.
#[embassy_executor::task]
pub async fn led_blink(mut led: Output<'static>, delay_ms: u64) {
  loop {
    led.toggle();
    Timing::delay_ms(delay_ms).await;
  }
}

/// Edge-triggered button monitor. Logs press/release transitions.
///
/// Assumes active-high wiring (matches [`crate::hardware::GpioDefaults::BUTTON_PULL`]
/// = `Pull::Down`): line goes high while pressed.
#[embassy_executor::task]
pub async fn button_monitor(button: Input<'static>) {
  let mut last_pressed = button.is_high();
  loop {
    let pressed = button.is_high();
    if pressed != last_pressed {
      if pressed {
        debug!("Button pressed!");
      } else {
        debug!("Button released!");
      }
      last_pressed = pressed;
    }
    Timing::delay_ms(Timing::BUTTON_DEBOUNCE_MS).await;
  }
}

/// RTC clock display task. Reads the RTC every minute and logs the time.
#[embassy_executor::task]
pub async fn rtc_clock(rtc: RtcTimeProvider) {
  loop {
    if let Ok(now) = rtc.now() {
      debug!(
        "RTC: {=u8:02}:{=u8:02}:{=u8:02}",
        now.hour(),
        now.minute(),
        now.second()
      );
    }
    Timing::delay_ms(60_000).await;
  }
}
