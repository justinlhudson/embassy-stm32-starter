use crate::hardware::{ButtonReader, LedControl, Timing};
use crate::*;
/// Task definitions and implementations
///
/// This module contains reusable Embassy tasks that can be
/// used across different binaries and applications.
use embassy_stm32::gpio::{Input, Output};
use embassy_stm32::rtc::Rtc;

/// LED blinking task - configurable blink rate
#[embassy_executor::task]
pub async fn led_blink(mut led: Output<'static>, delay_ms: u64) {
  loop {
    LedControl::turn_on(&mut led);
    Timing::delay_ms(delay_ms).await;

    LedControl::turn_off(&mut led);
    Timing::delay_ms(delay_ms).await;
  }
}

/// Button monitoring task
#[embassy_executor::task]
pub async fn button_monitor(button: Input<'static>) {
  let mut last_state = ButtonReader::is_released(&button);
  loop {
    let current_state = ButtonReader::is_pressed(&button);
    if current_state != last_state {
      if current_state {
        debug!("Button released!");
      } else {
        debug!("Button pressed!");
      }
      last_state = current_state;
    }
    Timing::delay_ms(Timing::BUTTON_DEBOUNCE_MS).await;
  }
}

/// System heartbeat task
#[embassy_executor::task]
pub async fn heartbeat_task() {
  let mut counter = 0;
  loop {
    counter += 1;
    info!("Heartbeat #{}", counter);
    Timing::delay_ms(Timing::HEARTBEAT_INTERVAL_MS).await;
  }
}

/// RTC clock display task
#[embassy_executor::task]
pub async fn rtc_clock(_rtc: Rtc) {
  let mut seconds: u64 = 0;
  loop {
    seconds = seconds.wrapping_add(1);
    if seconds % 10 == 0 {
      debug!("RTC seconds: {}", seconds);
    }
    Timing::delay_ms(Timing::RTC_UPDATE_INTERVAL_MS).await;
  }
}
