#[cfg(not(feature = "no-rtc"))]
use embassy_stm32::rtc::Rtc;
#[cfg(not(feature = "no-rtc"))]
/// RTC clock task - outputs current time to log
#[embassy_executor::task]
pub async fn rtc_clock(mut rtc: Rtc) {
  // Set initial datetime
  let initial_date = match NaiveDate::from_ymd_opt(2024, 1, 1) {
    Some(date) => date,
    None => {
      defmt::error!("Failed to create initial RTC date");
      return;
    }
  };
  let initial_time = match initial_date.and_hms_opt(12, 0, 0) {
    Some(dt) => dt,
    None => {
      defmt::error!("Failed to create initial RTC datetime");
      return;
    }
  };
  if let Err(e) = rtc.set_datetime(initial_time.into()) {
    defmt::error!("Failed to set RTC datetime: {:?}", e);
    return;
  }
  loop {
    let now: NaiveDateTime = match rtc.now() {
      Ok(val) => val.into(),
      Err(e) => {
        defmt::error!("RTC read error: {:?}", e);
        TimingUtils::delay_ms(TimingUtils::RTC_UPDATE_INTERVAL_MS).await;
        continue;
      }
    };
    let timestamp = now.and_utc().timestamp();
    info!(
      "RTC: {}-{:02}-{:02} {:02}:{:02}:{:02} (ts: {})",
      now.year(),
      now.month(),
      now.day(),
      now.hour(),
      now.minute(),
      now.second(),
      timestamp
    );
    TimingUtils::delay_ms(TimingUtils::RTC_UPDATE_INTERVAL_MS).await;
  }
}
use crate::hardware::{ButtonReader, LedControl, TimingUtils};
use crate::*;
#[cfg(not(feature = "no-rtc"))]
use chrono::{Datelike, NaiveDate, NaiveDateTime, Timelike};
/// Task definitions and implementations
///
/// This module contains reusable Embassy tasks that can be
/// used across different binaries and applications.
use embassy_stm32::gpio::{Input, Output};

/// LED blinking task - configurable blink rate
#[embassy_executor::task]
pub async fn led_blink(mut led: Output<'static>, delay_ms: u64) {
  loop {
    LedControl::turn_on(&mut led);
    TimingUtils::delay_ms(delay_ms).await;

    LedControl::turn_off(&mut led);
    TimingUtils::delay_ms(delay_ms).await;
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
        info!("Button pressed!");
      } else {
        info!("Button released!");
      }
      last_state = current_state;
    }
    TimingUtils::delay_ms(TimingUtils::BUTTON_DEBOUNCE_MS).await;
  }
}

/// System heartbeat task
#[embassy_executor::task]
pub async fn heartbeat_task() {
  let mut counter = 0;
  loop {
    counter += 1;
    info!("Heartbeat #{}", counter);
    TimingUtils::delay_ms(TimingUtils::HEARTBEAT_INTERVAL_MS).await;
  }
}

use embassy_time::Instant;

/// Timer-based clock task - outputs uptime to log
#[embassy_executor::task]
pub async fn timer_clock_task() {
  let mut last = Instant::now();
  let mut seconds: u64 = 0;
  loop {
    let now = Instant::now();
    let elapsed = now.duration_since(last).as_secs();
    if elapsed > 0 {
      seconds += elapsed;
      last = now;
      info!("Timer clock: {} seconds uptime", seconds);
    }
    TimingUtils::delay_ms(TimingUtils::RTC_UPDATE_INTERVAL_MS).await;
  }
}
