/// Timer Hardware Abstraction Layer
///
/// This module provides convenient abstractions for timer operations
/// and timing utilities for the STM32F446RE microcontroller.
use embassy_time::Timer;

/// Common timing utilities and constants
pub struct TimingUtils;

impl TimingUtils {
  /// Standard blink rates in milliseconds
  pub const FAST_BLINK_MS: u64 = 100;
  pub const MEDIUM_BLINK_MS: u64 = 250;
  pub const SLOW_BLINK_MS: u64 = 500;
  pub const VERY_SLOW_BLINK_MS: u64 = 1000;

  /// Button debounce delay
  pub const BUTTON_DEBOUNCE_MS: u64 = 50;

  /// Watchdog pet interval (should be less than watchdog timeout)
  pub const WATCHDOG_PET_MS: u64 = 250;

  /// Heartbeat interval
  pub const HEARTBEAT_INTERVAL_MS: u64 = 2000;

  /// RTC update interval
  pub const RTC_UPDATE_INTERVAL_MS: u64 = 1000;

  /// Async delay in milliseconds
  pub async fn delay_ms(ms: u64) {
    Timer::after_millis(ms).await;
  }
}
